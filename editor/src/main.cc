/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <cstdlib>
#include <iostream>
#include <string_view>

#include "qcl/basis.hh"

#include "qcl/vfs/vfs.hh"
#include "qcl/vfs/vfs_file.hh"
#include "qcl/vfs/vfs_embed.hh"
#include "qcl/animator.hh"
#include "qcl/types.hh"
#include "qcl/application.hh"
#include "qcl/dynamicLoad.hh"
#include "qstd/effects.hh"
#include "qstd/monet.hh"
#include "qstd/standard.hh"

#include "include/gpu/ganesh/SkSurfaceGanesh.h"
#include "include/gpu/ganesh/GrBackendSurface.h"
#include "include/gpu/ganesh/vk/GrVkBackendSurface.h"
#include "include/gpu/ganesh/vk/GrVkTypes.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkSurface.h"
#include "include/core/SkPaint.h"
#include "qcl/platform.hh"


using namespace std;
using namespace qcl;



#define DECLARE_SUB(Name, Type) Type* Name;
#define INIT_SUB(Name, Type) Name = dynamic_cast<Type*>(qcl::dyn::findFromName(this, #Name));
#define INIT_EV(Class, Name) {#Name, qev_seed(this, &Class::Name)},
#define SUBCOM_GENERATE(File, Class, Events, Comps) \
public: \
  Comps(DECLARE_SUB) \
  Class() { \
    qcl::dyn::loadFromData(this, File, {Events(INIT_EV)}); \
    Comps(INIT_SUB) \
    onCreate(this); \
  }



class MainForm: public qstd::form
{
public:
  #define COMPONENTS(DO) \
    DO(TestVK, qstd::viewPortVK)


  #define EVENTS(DO) \

  
  SUBCOM_GENERATE("embed://design/Main.qdl"_vfs_ro, MainForm, EVENTS, COMPONENTS)

  ~MainForm() { onDestroy(this); }

};




extern "C" int qcl_entry2(ohid App_H)
{
  auto vfs_file = vfs::__file::get();
  auto vfs_embed = vfs::__embed::get();


  application App(App_H, true);


  dyn::reg("qstd", qstd::objs());
  dyn::reg("qstd", qstd::effs());


  qstd::Monet = qstd::monet(
    color::oklch(0.675, 0.138, 65),
    color::oklch(0.563, 0.038, 65),
    color::oklch(0.517, 0.052, 300),
    true
  );


  auto AForm = new MainForm();
  AForm->show();

  CurrentApp->run();
  return 0;
}

extern "C" void platform::qcl_error(string_view Msg)
{
  cerr << "qcl error: " << Msg << endl;
  exit(1);
}

extern "C" void platform::qcl_warning(string_view Msg)
{
  cerr << "qcl warning: " << Msg << endl;
}
