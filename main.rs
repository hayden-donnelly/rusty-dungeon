// Compile with:
// rustc -C panic=abort -C link-arg=-nostartfiles main.rs

#![feature(generic_const_exprs)]
#![feature(lang_items)]
#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn rust_eh_personality() {}

const TCGETS: u64 = 0x5401;
const TCSETS: u64 = 0x5402;
const ECHO: u32 = 0o0000010;
const ICANON: u32 = 0o0000002;
const ISIG: u32 = 0o0000001;
const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;

const F_GETFL: i32 = 3;
const F_SETFL: i32 = 4;
const O_NONBLOCK: i32 = 0o4000;

const SYSCALL_READ: u64 = 0;
const SYSCALL_WRITE: u64 = 1;
const SYSCALL_IOCTL: u64 = 16;
const SYSCALL_EXIT: u64 = 60;
const SYSCALL_FCNTL: u64 = 72;

const KEY_UP: u8 = 65;
const KEY_DOWN: u8 = 66;
const KEY_RIGHT: u8 = 67;
const KEY_LEFT: u8 = 68;
const KEY_QUIT: u8 = 113;
const KEY_CONTINUE: u8 = 32;
const ALL_KEYS: [u8; 6] = [KEY_UP, KEY_DOWN, KEY_RIGHT, KEY_LEFT, KEY_QUIT, KEY_CONTINUE];

const WALL_CHAR: char = '#';
const PLAYER_CHAR: char = '@';
const STAIRS_CHAR: char = 'S';
const KEY_CHAR: char = 'K';
const FLOOR_CHAR: char = '.';

#[repr(C)]
#[derive(Copy, Clone)]
struct Termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_line: u8,
    c_cc: [u8; 32],
    c_ispeed: u32,
    c_ospeed: u32,
}

enum Result<T> {
    Ok(T),
    Err(i32), // Store the raw errno
}

fn exit(status: i32) -> ! {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        {
            core::arch::asm!(
                "syscall",
                in("rax") SYSCALL_EXIT,
                in("rdi") status,
                options(noreturn)
            );
        }
    }
}

unsafe fn ioctl(fd: i32, op: u64, argp: *mut Termios) -> i32 {
    let mut ret: i32;
    
    #[cfg(target_arch = "x86_64")]
    {
        asm!(
            "syscall",
            in("rax") SYSCALL_IOCTL,
            in("rdi") fd,
            in("rsi") op,
            in("rdx") argp,
            out("rcx") _,
            out("r11") _,
            lateout("rax") ret,
            options(nostack)
        );
    }
    
    ret
}

unsafe fn fcntl(fd: i32, op: i32, arg: i32) -> i32 {
    let mut ret: i32;
    
    #[cfg(target_arch = "x86_64")]
    {
        asm!(
            "syscall",
            in("rax") SYSCALL_FCNTL,
            in("rdi") fd,
            in("rsi") op,
            in("rdx") arg,
            out("rcx") _,
            out("r11") _,
            lateout("rax") ret,
            options(nostack)
        );
    }
    
    ret
}

unsafe fn read(fd: i32, buf: *mut u8, count: usize) -> i32 {
    let mut ret: i32;

    #[cfg(target_arch = "x86_64")]
    {
        asm!(
            "syscall",
            in("rax") SYSCALL_READ,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            out("rcx") _,
            out("r11") _,
            lateout("rax") ret,
            options(nostack)
        );
    }

    ret
}

unsafe fn write(fd: i32, buf: *const u8, count: usize) -> i32 {
    let mut ret: i32;

    #[cfg(target_arch = "x86_64")]
    {
        asm!(
            "syscall",
            in("rax") SYSCALL_WRITE,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            out("rcx") _,
            out("r11") _,
            lateout("rax") ret,
            options(nostack)
        );
    }

    ret
}

fn print(s: &str) {
    unsafe {
        write(STDOUT_FILENO, s.as_bytes().as_ptr(), s.len());
    }
}

fn print_char(c: char) {
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    print(s);
}

fn set_non_blocking(fd: i32) -> Result<i32> {
    unsafe {
        // Get the current flags
        let flags = fcntl(fd, F_GETFL, 0);
        if flags < 0 {
            return Result::Err(-flags);
        }
        
        // Add O_NONBLOCK to current flags
        let ret = fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        if ret < 0 {
            return Result::Err(-ret);
        }
        
        Result::Ok(flags)
    }
}

fn set_blocking(fd: i32, orig_flags: i32) -> Result<()> {
    unsafe {
        // Restore the original flags
        let ret = fcntl(fd, F_SETFL, orig_flags);
        if ret < 0 {
            return Result::Err(-ret);
        }
        
        Result::Ok(())
    }
}

fn enable_raw_mode() -> Result<Termios> {
    unsafe {
        // Get the current terminal attributes
        let mut orig_termios: Termios = core::mem::zeroed();
        let ret = ioctl(STDIN_FILENO, TCGETS, &mut orig_termios);
        if ret < 0 {
            return Result::Err(-ret);
        }
        
        let mut raw = orig_termios;
        
        // Disable ECHO, ICANON, and ISIG
        raw.c_lflag &= !(ECHO | ICANON | ISIG);
        let ret = ioctl(STDIN_FILENO, TCSETS, &mut raw);
        if ret < 0 {
            return Result::Err(-ret);
        }
        
        Result::Ok(orig_termios)
    }
}

fn disable_raw_mode(orig_termios: &mut Termios) -> Result<()> {
    unsafe {
        let ret = ioctl(STDIN_FILENO, TCSETS, orig_termios);
        if ret < 0 {
            return Result::Err(-ret);
        }
        
        Result::Ok(())
    }
}

fn get_input() -> u8 {
    let mut buf = [0u8; 1];
    unsafe {
        read(STDIN_FILENO, buf.as_mut_ptr(), 1);
    }

    buf[0]
}

struct XorshiftRng {
    state: u64
}

impl XorshiftRng {
    fn new(seed: u64) -> Self {
        XorshiftRng { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
    	x ^= x << 13;
	    x ^= x >> 17;
	    x ^= x << 5;
        self.state = x;
        self.state
    }

    fn range(&mut self, max: u64) -> u64 {
        self.next() % max
    }

    fn range_with_min(&mut self, min: u64, max: u64) -> u64 {
        if min < max {
            self.range(max - min) + min
        } else {
            self.range(max)
        }
    }
}

fn random_symbol(rng: &mut XorshiftRng) -> char {
    let symbols = ['@', '#', '%', '&', '*', '+', '=', '_'];
    let idx = rng.range(symbols.len() as u64) as usize;
    symbols[idx]
}

fn draw_level_into_window(
    window: &mut [char], level: &[char], 
    player_x: usize, player_y: usize, 
    window_width: usize, window_height: usize,
    level_width: usize, level_height: usize
) {
    let level_x_start: usize = player_x - (window_width / 2);
    let level_y_start: usize = player_y - (window_height / 2);
    for y in 0..window_height {
        let window_row_start: usize = y * window_width;
        let level_row_start: usize = (level_y_start + y) * level_width;
        for x in 0..window_width {
            if level_y_start + y == player_y && level_x_start + x == player_x {
                window[window_row_start + x] = '@';
            } else { 
                window[window_row_start + x] = level[level_row_start + level_x_start + x];
            }
        }
    }
}

fn get_time() -> (usize, usize) {
    let mut timespec = [0u64; 2];

    unsafe {
        #[cfg(target_arch = "x86_64")]
        {
            asm!(
                "mov rax, 228",
                "mov rdi, 0",
                "syscall",
                in("rsi") timespec.as_mut_ptr(),
                out("rax") _,
                out("rdi") _,
            );
        }
    }

    (timespec[0] as usize, timespec[1] as usize) 
}

fn ns_to_ms(time_ns: usize) -> usize {
    time_ns / 1000000
}

fn get_time_diff_ms(first_sec: usize, first_ns: usize, second_sec: usize, second_ns: usize) -> usize {
    const MS_PER_SEC: usize = 1000;
    if second_sec == first_sec && second_ns > first_ns {
        ns_to_ms(second_ns - first_ns)
    } else if second_sec > first_sec {
        (second_sec - first_sec - 1) * MS_PER_SEC + ns_to_ms(second_ns) + (MS_PER_SEC - ns_to_ms(first_ns))
    } else {
        0
    }
}

fn clear_screen() {
    print("\x1B[2J\x1B[1;1H");
}

struct Dungeon<const MAP_WIDTH: usize, const MAP_HEIGHT: usize> 
where
    [(); MAP_WIDTH * MAP_HEIGHT]: Sized
{
    rng: XorshiftRng,
    map: [char; MAP_WIDTH * MAP_HEIGHT],
    min_rooms: u64,
    max_rooms: u64,
    min_room_dim: u64,
    max_room_dim: u64,
    spawn_x: u64,
    spawn_y: u64,
    window_width: u64,
    window_height: u64,
}

impl<const MAP_WIDTH: usize, const MAP_HEIGHT: usize> Dungeon<MAP_WIDTH, MAP_HEIGHT> 
where
    [(); MAP_WIDTH * MAP_HEIGHT]: Sized
{
    fn new(
        seed: u64, 
        min_rooms: u64, 
        max_rooms: u64, 
        min_room_dim: u64, 
        max_room_dim: u64, 
        window_width: u64, 
        window_height: u64
    ) -> Self {
        Dungeon {
            rng: XorshiftRng::new(seed),
            map: ['#'; MAP_WIDTH * MAP_HEIGHT],
            min_rooms: min_rooms,
            max_rooms: max_rooms,
            min_room_dim: min_room_dim,
            max_room_dim: max_room_dim,
            spawn_x: 0,
            spawn_y: 0,
            window_width: window_width,
            window_height: window_height 
        }
    }

    fn map_width(&self) -> usize {
        MAP_WIDTH
    }

    fn map_height(&self) -> usize {
        MAP_HEIGHT
    }

    fn map_size(&self) -> usize {
        MAP_WIDTH * MAP_HEIGHT 
    }

    fn place_floor_tile(&mut self, x: usize, y: usize) {
        if x < MAP_WIDTH && y < MAP_HEIGHT {
            self.map[y * MAP_WIDTH + x] = FLOOR_CHAR;
        }
    }

    fn generate(&mut self) {
        for i in 0..self.map_size() {
            self.map[i] = '#';
        }

        // num_rooms must be at least 3 to accomodate special tiles
        let num_rooms = {
            let num_rooms_rng = self.rng.range_with_min(self.min_rooms, self.max_rooms);
            if num_rooms_rng >= 3 {
                num_rooms_rng
            } else {
                3
            }
        };

        let mut stairs_x: u64 = 0;
        let mut stairs_y: u64 = 0;
        let mut key_x: u64 = 0;
        let mut key_y: u64 = 0;


        let mut prev_room_x: u64 = 0;
        let mut prev_room_y: u64 = 0;

        for i in 0..num_rooms {
            let room_half_width = self.rng.range_with_min(self.min_room_dim, self.max_room_dim) / 2;
            let room_half_height = self.rng.range_with_min(self.min_room_dim, self.max_room_dim) / 2;
            // window_width and window_height are used to create a buffer zone around the edges of
            // edges of the map so that all rooms can be accessed with the window inbounds and the 
            // player centered in the window
            let room_x = self.rng.range_with_min(
                self.window_width + room_half_width + 1, 
                (MAP_WIDTH as u64) - room_half_width - self.window_width
            );
            let room_y = self.rng.range_with_min(
                self.window_height + room_half_height + 1, 
                (MAP_HEIGHT as u64) - room_half_height - self.window_height
            );
            
            // Fill in room with walkable tiles
            for y in (room_y - room_half_height)..(room_y + room_half_height) {
                for x in (room_x - room_half_width)..(room_x + room_half_width) {
                    self.map[(y as usize) * MAP_WIDTH + (x as usize)] = '.';
                }
            }
            
            // Connect current room to previous room
            if prev_room_x != 0 && prev_room_y != 0 {
                let hall_x;
                if prev_room_x < room_x {
                    hall_x = room_x;
                    for x in prev_room_x..(room_x+1) {
                        self.map[(prev_room_y as usize) * MAP_WIDTH + (x as usize)] = '.';
                    }
                } else {
                    hall_x = prev_room_x;
                    for x in room_x..(prev_room_x+1) {
                        self.map[(room_y as usize) * MAP_WIDTH + (x as usize)] = '.';
                    }
                }
                if prev_room_y < room_y {
                    for y in prev_room_y..(room_y+1) {
                        self.map[(y as usize) * MAP_WIDTH + (hall_x as usize)] = '.';
                    }
                } else {
                    for y in room_y..(prev_room_y+1) {
                        self.map[(y as usize) * MAP_WIDTH + (hall_x as usize)] = '.';
                    }
                }
            }
            
            // Determine coordinates of special tiles
            match i {
                0 => {
                    // Set spawn to the center of the first room
                    self.spawn_x = room_x;
                    self.spawn_y = room_y;
                },
                1 => {
                    // Put stairs in the second room
                    loop {
                        stairs_y = self.rng.range_with_min(room_y - room_half_height, room_y + room_half_height);
                        stairs_x = self.rng.range_with_min(room_x - room_half_width, room_x + room_half_width);
                        // Make sure the stairs don't overlap the spawn
                        if stairs_y != self.spawn_y || stairs_x != self.spawn_x {
                            break;
                        }
                    }
                },
                2 => {
                    // Put key in the third room
                    loop {
                        key_y = self.rng.range_with_min(room_y - room_half_height, room_y + room_half_height);
                        key_x = self.rng.range_with_min(room_x - room_half_width, room_x + room_half_width);
                        // Make sure the key doesn't overlap the spawn or the stairs
                        if (key_y != stairs_y || key_x != stairs_x) && (key_y != self.spawn_y || key_x != self.spawn_x) {
                            break;
                        }
                    }
                },
                _ => ()
            }

            prev_room_x = room_x;
            prev_room_y = room_y;
        }
        
        // Set special tiles
        self.map[(stairs_y as usize) * MAP_WIDTH + (stairs_x as usize)] = STAIRS_CHAR;
        self.map[(key_y as usize) * MAP_WIDTH + (key_x as usize)] = KEY_CHAR;
    }

    fn check_collision(&self, x: u64, y: u64) -> char {
        self.map[(y as usize) * MAP_WIDTH + (x as usize)]
    }
}

fn loop_until_continue() {
    loop {
        let cur_key = get_input();
        if cur_key == KEY_CONTINUE {
            break;
        }
    }
}

fn game() {
    let clear_screen_enabled = true;
    if clear_screen_enabled {
        clear_screen();
    }
    let mut rng = XorshiftRng::new(10142341231);

    const window_width: usize = 40;
    const half_window_width: usize = window_width / 2 + 1;
    const window_height: usize = 20;
    const half_window_height: usize = window_height / 2 + 1;
    const window_size: usize = window_width * window_height;

    const level_width: usize = 100 + (2 * window_width);
    const level_height: usize = 100 + (2 * window_height);
    const level_size: usize = level_width * level_height;
    const min_rooms: u64 = 10;
    const max_rooms: u64 = 50;
    const min_room_dim: u64 = 5;
    const max_room_dim: u64 = 20;
    let mut dungeon = Dungeon::<level_width, level_height>::new(
        1232123123234, 
        min_rooms, max_rooms, 
        min_room_dim, max_room_dim, 
        window_width as u64, window_height as u64
    );

    let mut window: [char; window_size] = [FLOOR_CHAR; window_size];

    let mut player_x: usize = 0;
    let mut player_y: usize = 0;

    let mut player_x_dir: isize = 1;
    let (mut last_sec, mut last_ns) = get_time();
    let mut last_key: u8 = 0;
    let mut has_stairs_key: bool = false;
    let mut should_generate_dungeon = true;

    // Game loop
    loop {
        let (cur_sec, cur_ns) = get_time();
        
        if should_generate_dungeon {
            dungeon.generate();
            player_x = dungeon.spawn_x as usize;
            player_y = dungeon.spawn_y as usize;
            has_stairs_key = false;
            should_generate_dungeon = false;
        }

        let cur_key = get_input();
        if cur_key == KEY_QUIT {
            break;
        } else if ALL_KEYS.contains(&cur_key) {
            last_key = cur_key;
        }

        let time_diff_ms = get_time_diff_ms(last_sec, last_ns, cur_sec, cur_ns);
        if time_diff_ms >= 30 {
            last_sec = cur_sec;
            last_ns = cur_ns;
           
            let target_player_x;
            let target_player_y;
            match last_key {
                KEY_UP => {
                    target_player_x = player_x;
                    target_player_y = player_y - 1;
                },
                KEY_DOWN => {
                    target_player_x = player_x;
                    target_player_y = player_y + 1;
                },
                KEY_LEFT => {
                    target_player_x = player_x - 1;
                    target_player_y = player_y;
                }
                KEY_RIGHT => {
                    target_player_x = player_x + 1;
                    target_player_y = player_y;
                },
                _ => {
                    target_player_x = player_x;
                    target_player_y = player_y;
                }
            }

            // Check if player can move to target position
            let mut should_wait_for_continue = false;
            let target_tile = dungeon.check_collision(target_player_x as u64, target_player_y as u64);
            match target_tile {
                FLOOR_CHAR => {
                    player_x = target_player_x;
                    player_y = target_player_y;
                },
                STAIRS_CHAR => {
                    if has_stairs_key {
                        player_x = target_player_x;
                        player_y = target_player_y;
                        should_generate_dungeon = true;
                    } else {
                        print("You must find the key!");
                        should_wait_for_continue = true;
                    }
                },
                KEY_CHAR => {
                    player_x = target_player_x;
                    player_y = target_player_y;
                    print("You found the key!");
                    has_stairs_key = true;
                    should_wait_for_continue = true;
                    dungeon.place_floor_tile(player_x, player_y);
                }
                _ => ()
            }

            last_key = 0;

            draw_level_into_window(
                &mut window, &dungeon.map, 
                player_x, player_y, 
                window_width, window_height, 
                level_width, level_height
            );


            if should_wait_for_continue {
                loop_until_continue();
            } 

            if clear_screen_enabled {
                clear_screen();
            }
            
            for h in 0..window_height {
                let row_start: usize = h * window_width;
                for w in 0..window_width {
                    print_char(window[row_start + w]);
                }
                print("\n");
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            "and rsp, -16", // Align stack to 16 bytes
            options(nostack)
        );
    }   
    main();
}

fn main() -> ! {
    let result = enable_raw_mode();
    match result {
        Result::Ok(mut orig_termios) => {
            let set_nb_result = set_non_blocking(STDIN_FILENO);
            let orig_fcntl_flags;
            match set_nb_result {
                Result::Ok(flags) => orig_fcntl_flags = flags,
                Result::Err(err) => {
                    print("Failed to set input to non-blocking\n");
                    exit(1);
                }
            }
            game();
            
            match disable_raw_mode(&mut orig_termios) {
                Result::Ok(_) => print("Terminal returned to normal mode\n"),
                Result::Err(err) => {
                    print("Failed to restore terminal to normal mode\n");
                    exit(1);
                }
            }

            if let Result::Err(err) = set_blocking(STDIN_FILENO, orig_fcntl_flags) {
                print("Failed to restore input to blocking\n");
            }
        },
        Result::Err(err) => {
            print("Failed to set terminal to raw mode\n");
            exit(1);
        }
    }
    exit(0);
}
