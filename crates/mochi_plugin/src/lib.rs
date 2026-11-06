/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{collections::HashMap, path::Path, rc::Rc};
use libloading::{Library, Symbol};
use plugin::*;

mod plugin;


pub enum Error {
  FileNotFound,
  UnknownError,
  CouldNotLoaded,
  CorruptPlugin,
  PluginCrashed,
  IncompatiblePlugin,
  PluginNotFound,
}


pub struct PluginSystem {
  libraries: HashMap<String, Rc<Library>>,
  plugins: HashMap<u32, (MochiPlugin, Rc<Library>)>,
  idx: u32,
}


impl PluginSystem {
  pub fn new() -> Self {
    Self {
      libraries: HashMap::default(),
      plugins: HashMap::default(),
      idx: 0,
    }
  }

  
  pub extern "C" fn do_hook(_thing: HookThing, _hook: unsafe extern "C" fn()) { todo!() }


  pub fn load_plugin(&mut self, path: &Path) -> Result<(), Error> {
    // Canonic Path
    if !path.exists() {
      return Err(Error::FileNotFound);
    }

    let path = path.canonicalize()
      .map_err(|_| Error::UnknownError)?
      .into_string()
      .map_err(|_| Error::UnknownError)?;


    // Worker
    let worker = |lib: &Rc<Library>, idx: &mut u32, plugins: &mut HashMap<u32, (MochiPlugin, Rc<Library>)>| -> Result<(), Error> {
      // Getter
      let getter = unsafe { lib.get(MOCHI_PLUGIN_GET) }
        .map_err(|_| Error::CorruptPlugin)?
        as Symbol<MochiPluginGet>;
      
      
      // Manifest
      let manifest = unsafe { getter() };
      
      let manifest = if !manifest.is_null() && manifest.is_aligned() {
        unsafe {*manifest}.clone()
      } else {
        return Err(Error::PluginCrashed)
      };


      // Running
      if manifest.version != MOCHI_PLUGIN_VER {
        return Err(Error::IncompatiblePlugin);
      }

      *idx += 1;

      let pctx = PluginContext {
        plg_id: *idx,
        hook: Self::do_hook,
      };

      unsafe {
        (manifest.on_load)(&pctx);
      }

      
      // Plugins
      plugins.insert(*idx, (manifest, lib.to_owned())).map(|_| panic!());

      Ok(())
    };


    // Library
    use std::collections::hash_map::Entry::*;
    
    match self.libraries.entry(path.clone()) {
      Occupied(entry) => worker(entry.get(), &mut self.idx, &mut self.plugins)?,

      Vacant(entry) => {
        let lib = unsafe { Library::new(&path) }.map_err(|_| Error::CouldNotLoaded)?;

        worker(entry.insert(Rc::new(lib)), &mut self.idx, &mut self.plugins)?
      }
    };

    Ok(())
  }

  pub fn unload_plugin(&mut self, id: u32) -> Result<(), Error> {
    match self.plugins.remove_entry(&id) {
      None => Err(Error::PluginNotFound),

      Some((idx, (plg, lib))) => {
        let pctx = PluginContext {
          plg_id: idx,
          hook: Self::do_hook,
        };
        
        unsafe { (plg.on_unload)(&pctx) }

        if Rc::strong_count(&lib) <= 2 {
          let fkey = self.libraries.iter().find_map(|(key, it)| Rc::ptr_eq(&lib, &it).then(|| key)).cloned();

          if let Some(fkey) = fkey {
            self.libraries.remove(&fkey);
          }
        }

        Ok(())
      }
    }
  }

}
