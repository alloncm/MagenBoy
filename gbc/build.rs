fn main()
{
    println!("cargo:rustc-link-lib=static=Engine");
    println!("cargo:rustc-link-lib=static=User32");
    println!("cargo:rustc-link-search=native=./Dependencies/");
}