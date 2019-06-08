use winapi::shared::winerror;

pub type Result<T> = std::result::Result<T, winerror::HRESULT>;

pub mod barrier;
pub mod command_allocator;
pub mod command_list;
pub mod device;
pub mod dxgi;
pub mod heap;
pub mod queue;
pub mod resource;
pub mod sync;
