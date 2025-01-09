use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;

pub fn process_html_content(
    content: &str, 
    resources: &HashMap<String, (PathBuf, String)>,
    image_cache: &HashMap<String, String>,  // 存储 文件名 -> base64 的映射
) -> String {
    let img_regex = Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).unwrap();
    
    img_regex.replace_all(content, |caps: &regex::Captures| {
        let img_src = &caps[1];
        
        // 处理不同格式的图片路径
        let normalized_src = if img_src.starts_with("../") {
            img_src.trim_start_matches("../").to_string()
        } else {
            img_src.to_string()
        };
        
        // 1. 直接匹配
        if let Some(base64_data) = image_cache.get(&normalized_src) {
            return caps[0].replace(img_src, base64_data);
        }

        // 2. 尝试拼接 OEBPS 前缀
        let with_prefix = format!("OEBPS/{}", normalized_src);
        if let Some(base64_data) = image_cache.get(&with_prefix) {
            return caps[0].replace(img_src, base64_data);
        }

        // 3. 尝试文件名匹配
        let path_buf = PathBuf::from(&normalized_src);
        let file_name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&normalized_src);

        if let Some(base64_data) = image_cache.get(file_name) {
            return caps[0].replace(img_src, base64_data);
        }

        // 4. 尝试在资源中查找匹配的路径
        for (key, data) in image_cache.iter() {
            if key.ends_with(&normalized_src) {
                return caps[0].replace(img_src, data);
            }
        }

        println!("No match found for: {} (normalized: {})", img_src, normalized_src);
        println!("Available paths: {:?}", image_cache.keys().collect::<Vec<_>>());
        caps[0].to_string()
    }).into_owned()
}
