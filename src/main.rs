use console::{Emoji, style};

const ASCII: &str = r#"
                            __ 
   ______      _____  ___  / /_
  / ___/ | /| / / _ \/ _ \/ __/
 /__  /| |/ |/ /  __/  __/ /_  
/____/ |__/|__/\___/\___/\__/  "#;

fn main() {
    let logo = style(ASCII).magenta().bold();
    let candy = Emoji("🍬 ", "!");

    println!("{}{}", logo, candy);

    println!(
        "\n{}",
        style("— A blazing-fast code health analyzer :)")
            .italic()
            .cyan()
    );
    println!("{}", style("─".repeat(45)).dim());
}
