extern crate pf;

use pf::PastFile;

fn main() {
    let link = PastFile::delete("Some link ...").unwrap();
    println!("{}", link);
}
