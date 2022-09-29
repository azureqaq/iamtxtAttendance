use clap::{ App, crate_name, crate_authors};


fn main() {
    let _app = App::new(crate_name!())
        .about("a good demo!")
        .author(crate_authors!())
        .build();


}