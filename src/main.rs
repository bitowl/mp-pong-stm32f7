#![no_std]
#![no_main]
#![feature(compiler_builtins_lib)]
#![feature(alloc)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]
#![feature(const_fn)]

extern crate compiler_builtins;
extern crate r0;
#[macro_use] // To get the hprintf! macro from semi-hosting
extern crate stm32f7_discovery as stm32f7;
#[macro_use]
extern crate alloc;

mod fps;
mod graphics;
mod lcd; // use custom LCD implementation
mod network;
mod racket;

use core::cmp::max;
use core::cmp::min;
use core::ptr;
use embedded::interfaces::gpio::Gpio;
use lcd::Framebuffer;
use lcd::FramebufferL8;
use network::Client;
use network::Server;
use stm32f7::{board, embedded, interrupts, sdram, system_clock, touch, i2c};

const USE_DOUBLE_BUFFER: bool = true;
const ENABLE_FPS_OUTPUT: bool = false;
const PRINT_START_MESSAGE: bool = false;
//Background Colour
const BGCOLOR: lcd::Color = lcd::Color::rgb(0, 0, 0);

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        static __DATA_LOAD: u32;
        static __DATA_END: u32;
        static mut __DATA_START: u32;

        static mut __BSS_START: u32;
        static mut __BSS_END: u32;
    }

    let data_load = &__DATA_LOAD;
    let data_start = &mut __DATA_START;
    let data_end = &__DATA_END;

    let bss_start = &mut __BSS_START;
    let bss_end = &__BSS_END;

    // initializes the .data section
    // (copy the data segment initializers from flash to RAM)
    r0::init_data(data_start, data_end, data_load);
    // zeroes the .bss section
    r0::zero_bss(bss_start, bss_end);

    stm32f7::heap::init();

    // Initialize the floating point unit
    let scb = stm32f7::cortex_m::peripheral::scb_mut();
    scb.cpacr.modify(|v| v | 0b1111 << 20);

    main(board::hw());
}

fn main(hw: board::Hardware) -> ! {
    if PRINT_START_MESSAGE {
        hprintln!(
            "\n[38;5;40m[1m🔦 Flash complete! ✔️\n[38;5;45m🚀 Program started.(B[m"
        );
    }

    let board::Hardware {
        rcc,
        pwr,
        flash,
        fmc,
        ltdc,
        gpio_a,
        gpio_b,
        gpio_c,
        gpio_d,
        gpio_e,
        gpio_f,
        gpio_g,
        gpio_h,
        gpio_i,
        gpio_j,
        gpio_k,
        i2c_3,
        sai_2,
        syscfg,
        ethernet_mac,
        ethernet_dma,
        nvic,
        ..
    } = hw;
    interrupts::scope(
        nvic,
        |_| {},
        move |interrupt_table| {
            let mut gpio = Gpio::new(
                gpio_a,
                gpio_b,
                gpio_c,
                gpio_d,
                gpio_e,
                gpio_f,
                gpio_g,
                gpio_h,
                gpio_i,
                gpio_j,
                gpio_k,
            );

            system_clock::init(rcc, pwr, flash);

            // enable all gpio ports
            rcc.ahb1enr.update(|r| {
                r.set_gpioaen(true);
                r.set_gpioben(true);
                r.set_gpiocen(true);
                r.set_gpioden(true);
                r.set_gpioeen(true);
                r.set_gpiofen(true);
                r.set_gpiogen(true);
                r.set_gpiohen(true);
                r.set_gpioien(true);
                r.set_gpiojen(true);
                r.set_gpioken(true);
            });

            // init sdram (for display)
            sdram::init(rcc, fmc, &mut gpio);

            // init touch screen
            i2c::init_pins_and_clocks(rcc, &mut gpio);
            let mut i2c_3 = i2c::init(i2c_3);
            touch::check_family_id(&mut i2c_3).unwrap();

            let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
            lcd.set_background_color(lcd::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            });
            let mut framebuffer = FramebufferL8::new();
            framebuffer.init();
            lcd.framebuffer_addr = framebuffer.get_framebuffer_addr() as u32;
            lcd.backbuffer_addr = framebuffer.get_backbuffer_addr() as u32;

            if !USE_DOUBLE_BUFFER {
                lcd.swap_buffers();
            }
            lcd.swap_buffers();

            let should_draw_now = false;
            let should_draw_now_ptr = (&should_draw_now as *const bool) as usize;

            let interrupt_handler = interrupt_table
                .register(
                    interrupts::interrupt_request::InterruptRequest::LcdTft,
                    interrupts::Priority::P1,
                    move || {
                        unsafe {
                            let need_draw = ptr::read_volatile(should_draw_now_ptr as *mut bool);
                            if !need_draw {
                                if USE_DOUBLE_BUFFER {
                                    lcd.swap_buffers();
                                }
                                ptr::write_volatile(should_draw_now_ptr as *mut bool, true);
                            }
                        }
                        lcd.clr_line_interrupt();
                    },
                )
                .expect("LcdTft interrupt already used");

            run(&mut framebuffer, &mut i2c_3, should_draw_now_ptr)
        },
    )
}

fn run(framebuffer: &mut FramebufferL8, i2c_3: &mut i2c::I2C, should_draw_now_ptr: usize) -> ! {
    hprintln!("Start run()");
    //// INIT COMPLETE ////
    let mut fps = fps::init();
    fps.output_enabled = ENABLE_FPS_OUTPUT;

    

    //Create Player 1 Racket
    let racket_1 = racket::Racket::new(xpos_centre_p1, ypos_centre, ypos_centre);

    //Create Player 1 Racket
    let racket_2 = racket::Racket::new(xpos_centre_p2, ypos_centre, ypos_centre);

    let mut rackets: [racket::Racket; 2] = [racket_1, racket_2];
     for racket in rackets.iter_mut() {racket.draw_racket_start_pos();}

    // setup local "network"
    let client1 = network::LocalClient::new();
    let client2 = network::LocalClient::new();
    let server = network::LocalServer::new();
    let server_gamestate = network::GamestatePacket::new();

    loop {
        let mut need_draw = false; // This memory space is accessed directly to achive synchronisation. Very unsafe!
        unsafe {
            // Frame synchronisation
            need_draw = ptr::read_volatile(should_draw_now_ptr as *mut bool);
        }
        if need_draw {
            if USE_DOUBLE_BUFFER {
                framebuffer.swap_buffers();
            }

            game_loop(
                &mut running_x,
                &mut running_y,
                framebuffer,
                &mut current_color,
                i2c_3,
                &fps,
                &mut rackets,
                &mut client1,
                &mut client2,
                &mut server,
                &mut server_gamestate,
            );

            // end of frame
            fps.count_frame();
            unsafe {
                ptr::write_volatile(should_draw_now_ptr as *mut bool, false);
            }
        }
    }
}

fn game_loop(
    running_x: &mut usize,
    running_y: &mut usize,
    framebuffer: &mut FramebufferL8,
    current_color: &mut u8,
    i2c_3: &mut i2c::I2C,
    fps: &fps::FpsCounter,
    rackets: &mut [racket::Racket; 2],
    client1: &mut Client,
    client2: &mut Client,
    server: &mut Server,
    server_gamestate: &mut GamestatePacket,
) {
    if is_server {
        let inputs = server.receive_inputs();
        calcute_physics(server_gamestate, inputs);
        server.send_gamestate(server_gamestate);
    }
    network::handle_local(client1, client2, server);

    input::input.evaluate_touch();
    client1.send_input();
    client2.send_input();
    let gamestate = client1.receive_gamestate();

    //move rackets and ball
    update_graphics(gamestate);
    graphics::draw_fps(framebuffer, fps);
}