/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub use mochi_rhi::*;


pub type Backend = mochi_rhi_ash::AshVulkanBackend;

pub type Device = mochi_rhi::Device<Backend>;
pub type Allocator = mochi_rhi::Allocator<Backend>;
pub type Buffer = mochi_rhi::Buffer<Backend>;
pub type Image2 = mochi_rhi::Image2<Backend>;
pub type ImageView2<'a> = mochi_rhi::ImageView2<'a, Backend>;
pub type Sampler2 = mochi_rhi::Sampler2<Backend>;
pub type CommandManager = mochi_rhi::CommandManager<Backend>;
pub type Command = mochi_rhi::Command<Backend>;
pub type Swapchain = mochi_rhi::Swapchain<Backend>;
pub type Surface = mochi_rhi::Surface<Backend>;
pub type Pipeline = mochi_rhi::Pipeline<Backend>;
pub type PipelineMeta = mochi_rhi::PipelineMeta<Backend>;
pub type Shader = mochi_rhi::Shader<Backend>;
pub type Synchronizer = mochi_rhi::Synchronizer<Backend>;
pub type Uploader = mochi_rhi::Uploader<Backend>;
pub type RenderTarget<'a> = mochi_rhi::RenderTarget<'a, Backend>;
pub type Semaphore = <Backend as mochi_rhi::RhiBackend>::Semaphore;
pub type Fence = <Backend as mochi_rhi::RhiBackend>::Fence;
