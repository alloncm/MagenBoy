/*extern crate winapi;
extern crate wchar;
use winapi::shared::minwindef::HINSTANCE;
use winapi::ctypes::wchar_t;
use wchar::wch_c;
*/

extern "C"
{
    fn InitLib();
    fn DrawCycle(colors:*mut u32, height:u32, width:u32)->i32;
}

fn main()
{
    unsafe
    {
        InitLib();
        /*
        let colors:[u32;50*50] = [0x50505050;50*50];

        loop
        {
            DrawCycle(colors[0] as *mut u32, 50, 50);
        }
        */
    }
}