/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Format {
	Undefined = 0,

	/// 8-bit
	Int8U = 13, // FORMAT_R8_UINT
	Int8S = 14,  // FORMAT_R8_SINT
	Norm8U = 9,  // FORMAT_R8_UNORM
	Norm8S = 10, // FORMAT_R8_SNORM

	V2Int8U = 20,  // FORMAT_R8G8_UINT
	V2Int8S = 21,  // FORMAT_R8G8_SINT
	V2Norm8U = 16, // FORMAT_R8G8_UNORM
	V2Norm8S = 17, // FORMAT_R8G8_SNORM

	V3Int8U = 27,  // FORMAT_R8G8B8_UINT
	V3Int8S = 28,  // FORMAT_R8G8B8_SINT
	V3Norm8U = 23, // FORMAT_R8G8B8_UNORM
	V3Norm8S = 24, // FORMAT_R8G8B8_SNORM

	V4Int8U = 41,  // FORMAT_R8G8B8A8_UINT
	V4Int8S = 42,  // FORMAT_R8G8B8A8_SINT
	V4Norm8U = 37, // FORMAT_R8G8B8A8_UNORM
	V4Norm8S = 38, // FORMAT_R8G8B8A8_SNORM

	/// 16-bit
	Int16U = 74, // FORMAT_R16_UINT
	Int16S = 75,  // FORMAT_R16_SINT
	Norm16U = 70, // FORMAT_R16_UNORM
	Norm16S = 71, // FORMAT_R16_SNORM
	Float16 = 76, // FORMAT_R16_SFLOAT

	V2Int16U = 81,  // FORMAT_R16G16_UINT
	V2Int16S = 82,  // FORMAT_R16G16_SINT
	V2Norm16U = 77, // FORMAT_R16G16_UNORM
	V2Norm16S = 78, // FORMAT_R16G16_SNORM
	V2Float16 = 83, // FORMAT_R16G16_SFLOAT

	V3Int16U = 88,  // FORMAT_R16G16B16_UINT
	V3Int16S = 89,  // FORMAT_R16G16B16_SINT
	V3Norm16U = 84, // FORMAT_R16G16B16_UNORM
	V3Norm16S = 85, // FORMAT_R16G16B16_SNORM
	V3Float16 = 90, // FORMAT_R16G16B16_SFLOAT

	V4Int16U = 95,  // FORMAT_R16G16B16A16_UINT
	V4Int16S = 96,  // FORMAT_R16G16B16A16_SINT
	V4Norm16U = 91, // FORMAT_R16G16B16A16_UNORM
	V4Norm16S = 92, // FORMAT_R16G16B16A16_SNORM
	V4Float16 = 97, // FORMAT_R16G16B16A16_SFLOAT

	/// 32-bit
	Int32U = 98, // FORMAT_R32_UINT
	Int32S = 99,   // FORMAT_R32_SINT
	Float32 = 100, // FORMAT_R32_SFLOAT

	V2Int32U = 101,  // FORMAT_R32G32_UINT
	V2Int32S = 102,  // FORMAT_R32G32_SINT
	V2Float32 = 103, // FORMAT_R32G32_SFLOAT

	V3Int32U = 104,  // FORMAT_R32G32B32_UINT
	V3Int32S = 105,  // FORMAT_R32G32B32_SINT
	V3Float32 = 106, // FORMAT_R32G32B32_SFLOAT

	V4Int32U = 107,  // FORMAT_R32G32B32A32_UINT
	V4Int32S = 108,  // FORMAT_R32G32B32A32_SINT
	V4Float32 = 109, // FORMAT_R32G32B32A32_SFLOAT

	// 64-bit
	Int64U = 110,  // FORMAT_R64_UINT
	Int64S = 111,  // FORMAT_R64_SINT
	Float64 = 112, // FORMAT_R64_SFLOAT

	V2Int64U = 113,  // FORMAT_R64G64_UINT
	V2Int64S = 114,  // FORMAT_R64G64_SINT
	V2Float64 = 115, // FORMAT_R64G64_SFLOAT

	V3Int64U = 116,  // FORMAT_R64G64B64_UINT
	V3Int64S = 117,  // FORMAT_R64G64B64_SINT
	V3Float64 = 118, // FORMAT_R64G64B64_SFLOAT

	V4Int64U = 119,  // FORMAT_R64G64B64A64_UINT
	V4Int64S = 120,  // FORMAT_R64G64B64A64_SINT
	V4Float64 = 121, // FORMAT_R64G64B64A64_SFLOAT

	// Image/Texture
	RGBA8Srgb = 43,  // FORMAT_R8G8B8A8_SRGB
	BGRA8Unorm = 44, // FORMAT_B8G8R8A8_UNORM
	BGRA8Srgb = 50,  // FORMAT_B8G8R8A8_SRGB

	// Depth/Stencil
	Depth16 = 124,          // FORMAT_D16_UNORM
	Depth32f = 126,         // FORMAT_D32_SFLOAT
	Depth24Stencil8 = 129,  // FORMAT_D24_UNORM_S8_UINT
	Depth32fStencil8 = 130, // FORMAT_D32_SFLOAT_S8_UINT
}

impl Format {
	pub fn from_raw(raw: u32) -> Self {
		match raw {
			0 => Format::Undefined,
			13 => Format::Int8U,
			14 => Format::Int8S,
			9 => Format::Norm8U,
			10 => Format::Norm8S,
			20 => Format::V2Int8U,
			21 => Format::V2Int8S,
			16 => Format::V2Norm8U,
			17 => Format::V2Norm8S,
			27 => Format::V3Int8U,
			28 => Format::V3Int8S,
			23 => Format::V3Norm8U,
			24 => Format::V3Norm8S,
			41 => Format::V4Int8U,
			42 => Format::V4Int8S,
			37 => Format::V4Norm8U,
			38 => Format::V4Norm8S,
			74 => Format::Int16U,
			75 => Format::Int16S,
			70 => Format::Norm16U,
			71 => Format::Norm16S,
			76 => Format::Float16,
			81 => Format::V2Int16U,
			82 => Format::V2Int16S,
			77 => Format::V2Norm16U,
			78 => Format::V2Norm16S,
			83 => Format::V2Float16,
			88 => Format::V3Int16U,
			89 => Format::V3Int16S,
			84 => Format::V3Norm16U,
			85 => Format::V3Norm16S,
			90 => Format::V3Float16,
			95 => Format::V4Int16U,
			96 => Format::V4Int16S,
			91 => Format::V4Norm16U,
			92 => Format::V4Norm16S,
			97 => Format::V4Float16,
			98 => Format::Int32U,
			99 => Format::Int32S,
			100 => Format::Float32,
			101 => Format::V2Int32U,
			102 => Format::V2Int32S,
			103 => Format::V2Float32,
			104 => Format::V3Int32U,
			105 => Format::V3Int32S,
			106 => Format::V3Float32,
			107 => Format::V4Int32U,
			108 => Format::V4Int32S,
			109 => Format::V4Float32,
			110 => Format::Int64U,
			111 => Format::Int64S,
			112 => Format::Float64,
			113 => Format::V2Int64U,
			114 => Format::V2Int64S,
			115 => Format::V2Float64,
			116 => Format::V3Int64U,
			117 => Format::V3Int64S,
			118 => Format::V3Float64,
			119 => Format::V4Int64U,
			120 => Format::V4Int64S,
			121 => Format::V4Float64,
			43 => Format::RGBA8Srgb,
			44 => Format::BGRA8Unorm,
			50 => Format::BGRA8Srgb,
			124 => Format::Depth16,
			126 => Format::Depth32f,
			129 => Format::Depth24Stencil8,
			130 => Format::Depth32fStencil8,
			_ => Format::Undefined,
		}
	}
}



pub struct VT {
	size: u64,
	align: u64,
	format: Format,
	count: u8,
}

impl VT {
	pub fn new(size: u64, align: u64, format: Format, count: u8) -> Self {
		Self {
			size,
			align,
			format,
			count,
		}
	}

	/*
	template <typename T>
	fun make() -> vt {
		// primitive
		if constexpr (std::is_same_v<T, u8>)  return {1, 1, Format::int8U};
		ef constexpr (std::is_same_v<T, u16>) return {2, 2, Format::int16U};
		ef constexpr (std::is_same_v<T, u32>) return {4, 4, Format::int32U};
		ef constexpr (std::is_same_v<T, u64>) return {8, 8, Format::int64U};

		ef constexpr (std::is_same_v<T, i8>)  return {1, 1, Format::int8S};
		ef constexpr (std::is_same_v<T, i16>) return {2, 2, Format::int16S};
		ef constexpr (std::is_same_v<T, i32>) return {4, 4, Format::int32S};
		ef constexpr (std::is_same_v<T, i64>) return {8, 8, Format::int64S};

		ef constexpr (std::is_same_v<T, nu8>)  return {1, 1, Format::norm8U};
		ef constexpr (std::is_same_v<T, nu16>) return {2, 2, Format::norm16U};

		ef constexpr (std::is_same_v<T, ni8>)  return {1, 1, Format::norm8S};
		ef constexpr (std::is_same_v<T, ni16>) return {2, 2, Format::norm16S};

		ef constexpr (std::is_same_v<T, f16>) return {2, 2, Format::float16};
		ef constexpr (std::is_same_v<T, f32>) return {4, 4, Format::float32};
		ef constexpr (std::is_same_v<T, f64>) return {8, 8, Format::float64};


		// vec2
		ef constexpr (std::is_same_v<T, vec2<u8>>)  return {2,   2,  Format::v2int8U};
		ef constexpr (std::is_same_v<T, vec2<u16>>) return {4,   4,  Format::v2int16U};
		ef constexpr (std::is_same_v<T, vec2<u32>>) return {8,   8,  Format::v2int32U};
		ef constexpr (std::is_same_v<T, vec2<u64>>) return {16,  16, Format::v2int64U};

		ef constexpr (std::is_same_v<T, vec2<i8>>)  return {2,   2,  Format::v2int8S};
		ef constexpr (std::is_same_v<T, vec2<i16>>) return {4,   4,  Format::v2int16S};
		ef constexpr (std::is_same_v<T, vec2<i32>>) return {8,   8,  Format::v2int32S};
		ef constexpr (std::is_same_v<T, vec2<i64>>) return {16,  16, Format::v2int64S};

		ef constexpr (std::is_same_v<T, vec2<nu8>>)  return {2,   2,  Format::v2norm8U};
		ef constexpr (std::is_same_v<T, vec2<nu16>>) return {4,   4,  Format::v2norm16U};

		ef constexpr (std::is_same_v<T, vec2<ni8>>)  return {2,   2,  Format::v2norm8S};
		ef constexpr (std::is_same_v<T, vec2<ni16>>) return {4,   4,  Format::v2norm16S};

		ef constexpr (std::is_same_v<T, vec2<f16>>) return {4,   4,  Format::v2float16};
		ef constexpr (std::is_same_v<T, vec2<f32>>) return {8,   8,  Format::v2float32};
		ef constexpr (std::is_same_v<T, vec2<f64>>) return {16,  16, Format::v2float64};


		// vec3
		ef constexpr (std::is_same_v<T, vec3<u8>>)  return {3,   4,  Format::v3int8U};
		ef constexpr (std::is_same_v<T, vec3<u16>>) return {6,   8,  Format::v3int16U};
		ef constexpr (std::is_same_v<T, vec3<u32>>) return {12,  16, Format::v3int32U};
		ef constexpr (std::is_same_v<T, vec3<u64>>) return {24,  32, Format::v3int64U};

		ef constexpr (std::is_same_v<T, vec3<i8>>)  return {3,   4,  Format::v3int8S};
		ef constexpr (std::is_same_v<T, vec3<i16>>) return {6,   8,  Format::v3int16S};
		ef constexpr (std::is_same_v<T, vec3<i32>>) return {12,  16, Format::v3int32S};
		ef constexpr (std::is_same_v<T, vec3<i64>>) return {24,  32, Format::v3int64S};

		ef constexpr (std::is_same_v<T, vec3<nu8>>)  return {3,   4,  Format::v3norm8U};
		ef constexpr (std::is_same_v<T, vec3<nu16>>) return {6,   8,  Format::v3norm16U};

		ef constexpr (std::is_same_v<T, vec3<ni8>>)  return {3,   4,  Format::v3norm8S};
		ef constexpr (std::is_same_v<T, vec3<ni16>>) return {6,   8,  Format::v3norm16S};

		ef constexpr (std::is_same_v<T, vec3<f16>>) return {6,   8,  Format::v3float16};
		ef constexpr (std::is_same_v<T, vec3<f32>>) return {12,  16, Format::v3float32};
		ef constexpr (std::is_same_v<T, vec3<f64>>) return {24,  32, Format::v3float64};


		// vec4
		ef constexpr (std::is_same_v<T, vec4<u8>>)  return {4,   4,  Format::v4int8U};
		ef constexpr (std::is_same_v<T, vec4<u16>>) return {8,   8,  Format::v4int16U};
		ef constexpr (std::is_same_v<T, vec4<u32>>) return {16,  16, Format::v4int32U};
		ef constexpr (std::is_same_v<T, vec4<u64>>) return {32,  32, Format::v4int64U};

		ef constexpr (std::is_same_v<T, vec4<i8>>)  return {4,   4,  Format::v4int8S};
		ef constexpr (std::is_same_v<T, vec4<i16>>) return {8,   8,  Format::v4int16S};
		ef constexpr (std::is_same_v<T, vec4<i32>>) return {16,  16, Format::v4int32S};
		ef constexpr (std::is_same_v<T, vec4<i64>>) return {32,  32, Format::v4int64S};

		ef constexpr (std::is_same_v<T, vec4<nu8>>)  return {4,   4,  Format::v4norm8U};
		ef constexpr (std::is_same_v<T, vec4<nu16>>) return {8,   8,  Format::v4norm16U};

		ef constexpr (std::is_same_v<T, vec4<ni8>>)  return {4,   4,  Format::v4norm8S};
		ef constexpr (std::is_same_v<T, vec4<ni16>>) return {8,   8,  Format::v4norm16S};

		ef constexpr (std::is_same_v<T, vec4<f16>>) return {8,   8,  Format::v4float16};
		ef constexpr (std::is_same_v<T, vec4<f32>>) return {16,  16, Format::v4float32};
		ef constexpr (std::is_same_v<T, vec4<f64>>) return {32,  32, Format::v4float64};


		// mat4x4
		ef constexpr (std::is_same_v<T, mat4x4<f32>>) return {16, 16, Format::v4float32, 4};
		ef constexpr (std::is_same_v<T, mat4x4<f64>>) return {32, 32, Format::v4float64, 4};

		// mat4x3
		ef constexpr (std::is_same_v<T, mat4x3<f32>>) return {16, 16, Format::v4float32, 3};
		ef constexpr (std::is_same_v<T, mat4x3<f64>>) return {32, 32, Format::v4float64, 3};
		else
			static_assert(false, "unsupported type");
	}


	/// @brief Create a list of vt instances for the given types.
	template <typename... Ts>
	static fun make_list() -> std::vector<vt> {
		return { vt::make<Ts>()... };
	}
	*/

	pub fn size(&self) -> u64 {
		self.size
	}

	pub fn align(&self) -> u64 {
		self.align
	}

	pub fn format(&self) -> Format {
		self.format
	}

	pub fn count(&self) -> u8 {
		self.count
	}
}
