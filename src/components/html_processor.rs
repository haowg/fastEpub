use std::collections::HashMap;
use std::path::PathBuf;

pub fn process_html_content(content: &str, images: &HashMap<String, String>, root_path: &PathBuf) -> String {
    let re = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).unwrap();
    
    re.replace_all(content, |caps: &regex::Captures| {
        let img_src = &caps[1];
        let img_path = if img_src.starts_with("data:") {
            img_src.to_string()
        } else {
            // 尝试多种路径组合来匹配图片
            let possible_paths = vec![
                img_src.to_string(),
                if img_src.starts_with('/') { 
                    img_src[1..].to_string()
                } else { 
                    img_src.to_string() 
                },
                format!("{}", img_src.split('/').last().unwrap_or(img_src)),
            ];

            // 尝试所有可能的路径并返回找到的图片路径或默认图片
            let default_image = String::from("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA...");
            match possible_paths.iter()
                .find(|path| images.contains_key(*path))
                .and_then(|path| images.get(path)) {
                    Some(found_image) => found_image.clone(),
                    None => {
                        println!("Image not found: {}", img_src);
                        default_image
                    }
                }
        };
        
        format!(r#"<img src="{}" loading="lazy""#, img_path)
    }).to_string()
}
