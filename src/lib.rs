use jay_config::{
    get_workspace,
    keyboard::{
        mods::SHIFT,
        parse_keymap,
        syms::{
            KeySym, SYM_Escape, SYM_Return, SYM_a, SYM_b, SYM_c, SYM_e, SYM_i, SYM_l, SYM_m, SYM_n,
            SYM_r, SYM_s, SYM_t, SYM_u, SYM_w,
        },
    },
};

use {
    chrono::{format::StrftimeItems, Local},
    jay_config::{
        config,
        exec::Command,
        input::{get_seat, input_devices, on_new_input_device, InputDevice, Seat},
        keyboard::syms::{SYM_d, SYM_f, SYM_q, SYM_F1},
        quit,
        status::set_status,
        timer::{duration_until_wall_clock_is_multiple_of, get_timer},
        video::{get_connector, on_connector_connected, on_graphics_initialized, on_new_connector},
        Direction::{Down, Left, Right, Up},
    },
    std::time::Duration,
};

fn configure_seat(s: Seat) {
    s.bind(SYM_F1, move || {
        s.bind(SYM_q, quit);
        s.bind(SYM_w, move || {
            s.close();
            unbind(s);
        });
        s.bind(SYM_f, move || {
            s.toggle_fullscreen();
            unbind(s);
        });

        s.bind(SYM_n, move || {
            s.focus(Left);
            unbind(s);
        });
        s.bind(SYM_e, move || {
            s.focus(Down);
            unbind(s);
        });
        s.bind(SYM_u, move || {
            s.focus(Up);
            unbind(s);
        });
        s.bind(SYM_i, move || {
            s.focus(Right);
            unbind(s);
        });

        s.bind(SYM_d, move || {
            s.bind(SYM_Return, move || {
                Command::new("alacritty")
                    .arg("-e")
                    .arg("toolbox")
                    .arg("run")
                    .arg("--container")
                    .arg("archlinux-toolbox-latest")
                    .arg("fish")
                    .spawn();
                unbind(s);
            });

            s.bind(SYM_b, move || {
                Command::new("flatpak")
                    .arg("run")
                    .arg("org.mozilla.firefox")
                    .spawn();
                unbind(s);
            });

            s.bind(SYM_d, move || {
                Command::new("rofi")
                    .arg("-combi-modi")
                    .arg("window,drun")
                    .arg("-show")
                    .arg("combi")
                    .arg("-show-icons")
                    .spawn();
                unbind(s);
            });
        });

        s.bind(SYM_l, move || {
            Command::new("~/.config/sway/power-menu").spawn();
            unbind(s);
        });

        let workspaces = [SYM_b, SYM_c, SYM_s, SYM_t, SYM_m];
        for (i, sym) in workspaces.into_iter().enumerate() {
            bind_workspace(s, sym, i + 1);
        }

        s.bind(SYM_Escape, move || {
            unbind(s);
        });
    });
}

fn unbind(s: Seat) {
    s.unbind(SYM_Escape);
    s.unbind(SYM_w);
    s.unbind(SYM_f);
    s.unbind(SYM_q);
    s.unbind(SYM_d);
    s.unbind(SYM_a);
    s.unbind(SYM_b);
    s.unbind(SYM_r);
    s.unbind(SYM_t);
    s.unbind(SYM_l);
    s.unbind(SYM_n);
    s.unbind(SYM_e);
    s.unbind(SYM_i);
    s.unbind(SYM_u);
    s.unbind(SYM_c);
    s.unbind(SYM_s);
    s.unbind(SYM_m);
    s.unbind(SYM_Return);
}

fn bind_workspace(s: Seat, sym: KeySym, name: usize) {
    let ws = get_workspace(&format!("{}", name));
    s.bind(sym, move || {
        s.show_workspace(ws);
        unbind(s);
    });
    s.bind(SHIFT | sym, move || {
        s.set_workspace(ws);
        unbind(s);
    });
}

fn setup_status() {
    let time_format: Vec<_> = StrftimeItems::new("%Y-%m-%d %H:%M").collect();
    let update_status = move || {
        let status = format!("{}", Local::now().format_with_items(time_format.iter()));
        set_status(&status);
    };
    update_status();
    let period = Duration::from_secs(5);
    let timer = get_timer("status_timer");
    timer.repeated(duration_until_wall_clock_is_multiple_of(period), period);
    timer.on_tick(update_status);
}


fn arrange_outputs() {
    let left = get_connector("eDP-1");
    let right = get_connector("DP-5");
    if left.connected() && right.connected() {
        left.set_position(0, 0);
        right.set_position(left.width(), 0);
    }
}


fn setup_outputs() {
    on_new_connector(move |_| arrange_outputs());
    on_connector_connected(move |_| arrange_outputs());
    arrange_outputs();
}

pub fn configure() {
    let seat = get_seat("default");
    seat.set_keymap(parse_keymap(include_str!("keymap.xkb")));

    configure_seat(seat);

    let handle_input_device = move |device: InputDevice| {
        device.set_seat(seat);
    };
    input_devices().into_iter().for_each(handle_input_device);
    on_new_input_device(handle_input_device);

    setup_status();
    setup_outputs();

    on_graphics_initialized(|| {
        Command::new("mako").spawn();
    });
}

config!(configure);
