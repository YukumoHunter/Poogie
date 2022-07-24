use anyhow::Result;
use ash::vk;

pub struct Swapchain {
    raw: vk::SwapchainKHR,
}