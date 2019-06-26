extern crate pf;

use pf::PastFile;

fn main() {
    let link = PastFile::create("Some Text ...").unwrap();
    println!("{}", link);
}
