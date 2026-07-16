#!/usr/bin/env python3

import os
import sys
import json
import struct
import subprocess
import tempfile

MAGIC = "\x1b«QAOS!»mochi-engine-asset-///".encode('utf-8')

class ChunkKind:
  Vertex      = 0x1
  Index       = 0x2
  LayerMeta   = 0x3
  LayerData   = 0x4
  TextureMeta = 0x5
  TextureData = 0x6
  Skinning    = 0x7
  Bone        = 0x8

  @classmethod
  def to_str(cls, val):
    for k, v in cls.__dict__.items():
      if v == val: return k
    return f"UNKNOWN (0x{val:x})"


class DataKind:
  Albedo    = 0x1
  Normal    = 0x2
  Depth     = 0x3
  Roughness = 0x4
  Occlusion = 0x5
  Emissive  = 0x6

  @classmethod
  def to_str(cls, val):
    for k, v in cls.__dict__.items():
      if v == val: return k
    return f"UNKNOWN (0x{val:x})"


class DataType:
  Map = 0x1
  Arr = 0x2

  @classmethod
  def to_str(cls, val):
    for k, v in cls.__dict__.items():
      if v == val: return k
    return f"UNKNOWN (0x{val:x})"


def format_bytes(size_in_bytes):
  units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  for unit in units:
    if size_in_bytes < 1024.0:
      return f"{size_in_bytes:.2f} {unit}"
    size_in_bytes /= 1024.0
  return f"{size_in_bytes:.2f} EB"


def align_bytes(data: bytes, alignment: int = 16) -> bytes:
  pad = (alignment - (len(data) % alignment)) % alignment
  return data + (b'\x00' * pad)


def read_glb_structures(glb_path):
  with open(glb_path, 'rb') as f:
    if f.read(4) != b'glTF':
      raise ValueError("Invalid GLB magic.")
    
    _ = struct.unpack('<I', f.read(4))[0]
    length = struct.unpack('<I', f.read(4))[0]
    
    # JSON chunk
    c0_len = struct.unpack('<I', f.read(4))[0]
    if f.read(4) != b'JSON':
      raise ValueError("Missing JSON chunk.")
    json_data = json.loads(f.read(c0_len).decode('utf-8'))
    
    # Binary chunk
    bin_buffer = b''
    if f.tell() < length:
      c1_len = struct.unpack('<I', f.read(4))[0]
      c1_type = f.read(4)
      if c1_type in (b'BIN\x00', b'BIN'):
        bin_buffer = f.read(c1_len)
        
  return json_data, bin_buffer


def parse_accessor(json_data, bin_buffer, accessor_idx):
  if accessor_idx is None:
    return b'', 0, 0
    
  accessor = json_data['accessors'][accessor_idx]
  bv = json_data['bufferViews'][accessor.get('bufferView', 0)]
  
  start = bv.get('byteOffset', 0) + accessor.get('byteOffset', 0)
  count = accessor['count']
  comp_type = accessor['componentType']
  
  comp_sizes = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
  type_counts = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}
  
  elem_size = comp_sizes[comp_type] * type_counts[accessor['type']]
  stride = bv.get('byteStride', elem_size)
  
  data = bytearray()
  for i in range(count):
    offset = start + i * stride
    data.extend(bin_buffer[offset:offset+elem_size])
  return bytes(data), count, comp_type


def run_toktx(raw_bytes, mime_type):
  ext = '.jpg' if mime_type == 'image/jpeg' else '.png'
  with tempfile.NamedTemporaryFile(delete=False, suffix=ext) as fin:
    fin.write(raw_bytes)
    fin_name = fin.name
    
  fout_name = fin_name + '.ktx2'
  try:
    cmd = ['toktx', '--2d', '--uastc', '3', fout_name, fin_name]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    with open(fout_name, 'rb') as f:
      return f.read()
  except Exception:
    return raw_bytes
  finally:
    if os.path.exists(fin_name): os.remove(fin_name)
    if os.path.exists(fout_name): os.remove(fout_name)


def process_glb_textures(json_data, bin_buffer):
  payloads, metas = [], []
  tex_kinds = {}
  
  for mat in json_data.get('materials', []):
    pbr = mat.get('pbrMetallicRoughness', {})
    if 'baseColorTexture' in pbr: tex_kinds[pbr['baseColorTexture']['index']] = DataKind.Albedo
    if 'normalTexture' in mat: tex_kinds[mat['normalTexture']['index']] = DataKind.Normal
    if 'metallicRoughnessTexture' in pbr: tex_kinds[pbr['metallicRoughnessTexture']['index']] = DataKind.Roughness
    if 'occlusionTexture' in mat: tex_kinds[mat['occlusionTexture']['index']] = DataKind.Occlusion
    if 'emissiveTexture' in mat: tex_kinds[mat['emissiveTexture']['index']] = DataKind.Emissive

  offset = 0
  for idx, tex in enumerate(json_data.get('textures', [])):
    img = json_data['images'][tex.get('source', 0)]
    bv = json_data['bufferViews'][img['bufferView']]
    raw_img = bin_buffer[bv.get('byteOffset', 0):bv.get('byteOffset', 0)+bv['byteLength']]
    
    ktx2_data = run_toktx(raw_img, img.get('mimeType', 'image/png'))
    size = len(ktx2_data)
    
    metas.append((tex_kinds.get(idx, DataKind.Albedo), DataType.Map, offset, size))
    payloads.append(ktx2_data)
    offset += size
    
    pad = (8 - (size % 8)) % 8
    payloads.append(b'\x00' * pad)
    offset += pad
    
  return b''.join(payloads), metas


def process_glb_geometry(json_data, bin_buffer):
  v_pay, i_pay, ld_pay = bytearray(), bytearray(), bytearray()
  layer_metas = []
  v_off, i_off, ld_off = 0, 0, 0
  
  for mesh in json_data.get('meshes', []):
    for prim in mesh.get('primitives', []):
      v_bytes, v_count, _ = parse_accessor(json_data, bin_buffer, prim['attributes'].get('POSITION'))
      if not v_bytes: continue
        
      i_bytes, i_count, comp_type = parse_accessor(json_data, bin_buffer, prim.get('indices'))
      if i_bytes:
        if comp_type == 5123:
          i_bytes = struct.pack(f'<{i_count}I', *struct.unpack(f'<{i_count}H', i_bytes))
        elif comp_type == 5121:
          i_bytes = struct.pack(f'<{i_count}I', *struct.unpack(f'<{i_count}B', i_bytes))
      else:
        i_bytes = struct.pack(f'<{v_count}I', *range(v_count))
        i_count = v_count
        
      mat_idx = prim.get('material')
      tex_ids = []
      if mat_idx is not None:
        mat = json_data['materials'][mat_idx]
        pbr = mat.get('pbrMetallicRoughness', {})
        
        # Corrected: Explicitly check all slots and filter duplicates
        slots = [
          pbr.get('baseColorTexture'),
          pbr.get('metallicRoughnessTexture'),
          mat.get('normalTexture'),
          mat.get('occlusionTexture'),
          mat.get('emissiveTexture')
        ]
        for slot in slots:
          if slot and 'index' in slot:
            t_idx = slot['index']
            if t_idx not in tex_ids:
              tex_ids.append(t_idx)
        
      ld_bytes = struct.pack(f'<{len(tex_ids)}I', *tex_ids)
      
      layer_metas.append({'i_off': i_off, 'i_size': len(i_bytes), 'ld_off': ld_off, 'ld_size': len(ld_bytes)})
      v_pay.extend(v_bytes)
      i_pay.extend(i_bytes)
      ld_pay.extend(ld_bytes)
      
      v_off += len(v_bytes)
      i_off += len(i_bytes)
      ld_off += len(ld_bytes)
      
  return bytes(v_pay), bytes(i_pay), bytes(ld_pay), layer_metas


def save_masset_package(masset_path, geometry_tuple, texture_tuple):
  v_raw, i_raw, ld_raw, layer_metas = geometry_tuple
  td_raw, texture_metas = texture_tuple
  
  v_chunk = align_bytes(v_raw)
  i_chunk = align_bytes(i_raw)
  ld_chunk = align_bytes(ld_raw)
  td_chunk = align_bytes(td_raw)
  
  base_offset = 200 
  abs_v = base_offset
  abs_i = abs_v + len(v_chunk)
  abs_ld = abs_i + len(i_chunk)
  abs_td = abs_ld + len(ld_chunk)
  
  lm_raw = b''.join([struct.pack('<QQQQ', m['i_off'], m['i_size'], m['ld_off'], m['ld_size']) for m in layer_metas])
  lm_chunk = align_bytes(lm_raw)
  abs_lm = abs_td + len(td_chunk)
  
  tm_raw = b''.join([struct.pack('<QQQ', tm[0] | (tm[1] << 56), tm[2], tm[3]) for tm in texture_metas])
  tm_chunk = align_bytes(tm_raw)
  abs_tm = abs_lm + len(lm_chunk)
  
  header = struct.pack('<32sIIQQ', MAGIC, 1, 1, 56, 6)
  toc = b''.join([
    struct.pack('<QQQ', ChunkKind.Vertex, abs_v, len(v_chunk)),
    struct.pack('<QQQ', ChunkKind.Index, abs_i, len(i_chunk)),
    struct.pack('<QQQ', ChunkKind.LayerMeta, abs_lm, len(lm_chunk)),
    struct.pack('<QQQ', ChunkKind.LayerData, abs_ld, len(ld_chunk)),
    struct.pack('<QQQ', ChunkKind.TextureMeta, abs_tm, len(tm_chunk)),
    struct.pack('<QQQ', ChunkKind.TextureData, abs_td, len(td_chunk))
  ])
  
  with open(masset_path, 'wb') as out:
    out.write(header)
    out.write(toc)
    out.write(v_chunk)
    out.write(i_chunk)
    out.write(ld_chunk)
    out.write(td_chunk)
    out.write(lm_chunk)
    out.write(tm_chunk)


def convert_glb_to_masset(glb_path, masset_path):
  if not os.path.exists(glb_path):
    print(f"Error: {glb_path} not found.")
    return False
  json_data, bin_buffer = read_glb_structures(glb_path)
  geometry_tuple = process_glb_geometry(json_data, bin_buffer)
  texture_tuple = process_glb_textures(json_data, bin_buffer)
  save_masset_package(masset_path, geometry_tuple, texture_tuple)
  print(f"Saved: {masset_path}")
  return True


def dump_masset(masset_path):
  if not os.path.exists(masset_path):
    print(f"Error: {masset_path} not found.")
    return False
    
  with open(masset_path, 'rb') as f:
    header = f.read(56)
    magic, ver, subver, toc_off, toc_count = struct.unpack('<32sIIQQ', header)
    
    print("Header:")
    print(f"  Version: {ver}.{subver}")
    print(f"  ChunkC : {toc_count}")
    print()
    
    f.seek(toc_off)
    chunks = []
    print("Chunks:")
    for i in range(toc_count):
      kind, off, size = struct.unpack('<QQQ', f.read(24))
      print(f"  #{i}  {ChunkKind.to_str(kind):<12}  {format_bytes(size):<8}  (0x{off:x}..{(off+size):x})")
      chunks.append({'kind': kind, 'off': off, 'size': size})
    print()
      
    ld_chunk = next((c for c in chunks if c['kind'] == ChunkKind.LayerData), None)

    for ch in chunks:
      if ch['kind'] == ChunkKind.LayerMeta:
        print("Layers:")
        f.seek(ch['off'])
        for l_idx in range(ch['size'] // 32):
          i_off, i_size, ld_off, ld_size = struct.unpack('<QQQQ', f.read(32))
          
          texture_ids_list = []
          if ld_chunk and ld_size > 0:
            saved_pos = f.tell()
            f.seek(ld_chunk['off'] + ld_off)
            num_ids = ld_size // 4
            texture_ids_list = list(struct.unpack(f'<{num_ids}I', f.read(ld_size)))
            f.seek(saved_pos)
            
          ids_str = ", ".join(map(str, texture_ids_list)) if texture_ids_list else "None"
          print(f"  #{l_idx}  IndexRange({i_off // 4}..{(i_off + i_size) // 4}), TexIDs([{ids_str}])")
        print()
        
      elif ch['kind'] == ChunkKind.TextureMeta:
        print("Textures:")
        f.seek(ch['off'])
        for t_idx in range(ch['size'] // 24):
          kt, off, size = struct.unpack('<QQQ', f.read(24))
          kind, dtype = kt & 0xFFFFFFFFFFFFFF, (kt >> 56) & 0xFF
          print(f"  #{t_idx}  {DataKind.to_str(kind):<9} ({DataType.to_str(dtype)})  {format_bytes(size)} (0x{off:x}..0x{(off+size):x})")
  return True


if __name__ == '__main__':
  if len(sys.argv) < 2:
    print("Usage:\n  ./masset.py convert <in.glb> <out.masset>\n  ./masset.py dump <in.masset>")
    sys.exit(1)
    
  mode = sys.argv[1].lower()
  if mode == 'convert' and len(sys.argv) >= 4:
    out_path = sys.argv[3] if sys.argv[3].endswith('.masset') else os.path.splitext(sys.argv[3])[0] + '.masset'
    convert_glb_to_masset(sys.argv[2], out_path)
  elif mode == 'dump' and len(sys.argv) >= 3:
    dump_masset(sys.argv[2])
  else:
    print("Invalid command arguments.")
