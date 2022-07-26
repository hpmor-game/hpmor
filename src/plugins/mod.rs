macro_rules! use_all {
    ($($p:ident),+) => {
        $(
            mod $p;
            pub use $p::*;
        )+
    };
}

use_all!(fullscreen_toggle, fps_meter, sprite_sorting);
