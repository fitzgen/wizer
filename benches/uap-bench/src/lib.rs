#[cfg(feature = "wizer")]
mod alloc;
#[cfg(feature = "wizer")]
use alloc::ZeroDuringInitAllocator;

use regex::RegexSet;
use serde::Deserialize;

#[cfg(feature = "wizer")]
#[global_allocator]
static ALLOC: ZeroDuringInitAllocator = ZeroDuringInitAllocator::new();

static mut UA_REGEX_SET: Option<RegexSet> = None;

#[derive(Deserialize)]
struct UserAgentParsers {
    user_agent_parsers: Vec<UserAgentParserEntry>,
}

#[derive(Deserialize)]
struct UserAgentParserEntry {
    regex: String,
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let uap_yaml = include_str!("../uap-core/regexes.yaml");
    let parsers: UserAgentParsers = serde_yaml::from_str(uap_yaml).unwrap();
    let regex_set = RegexSet::new(
        parsers
            .user_agent_parsers
            .iter()
            .map(|e| e.regex.replace("\\/", "/").replace("\\!", "!")),
    )
    .unwrap();
    unsafe {
        assert!(UA_REGEX_SET.is_none());
        UA_REGEX_SET = Some(regex_set);
    }

    #[cfg(feature = "wizer")]
    ALLOC.finish_init();
}

#[export_name = "run"]
pub extern "C" fn run(ptr: *mut u8, len: usize) -> i32 {
    #[cfg(not(feature = "wizer"))]
    init();

    let s = unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        std::str::from_utf8(slice).unwrap()
    };
    let regex_set = unsafe { UA_REGEX_SET.as_ref().unwrap() };
    regex_set.is_match(&s) as u8 as i32
}

#[export_name = "alloc"]
pub extern "C" fn alloc(size: usize, align: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
    unsafe { std::alloc::alloc(layout) }
}

#[export_name = "dealloc"]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize, align: usize) {
    let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
    unsafe {
        std::alloc::dealloc(ptr, layout);
    }
}
