use json_parser::parse;


fn main() {
    let json = parse::load_from_file("tree.json");
    println!("{:#?}", json);
}
