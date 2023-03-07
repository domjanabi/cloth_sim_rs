use olc_pge as olc;

pub const CHANGE_STATE_RADIUS: f32 = 40.0;
pub const CUT_RADIUS: f32 = 3.0;
pub const POINT_RADIUS: f32 = 2.5;
pub const HIGHLIGHT_RADIUS: f32 = 4.0;
pub const FORCE: f32 = 5.0;
pub const SNAP_RATIO: f32 = 3.0;

pub const STICK_COLOUR: olc::Pixel = olc::Pixel {
    r: 50,
    g: 80,
    b: 50,
    a: 255,
};
pub const HIGHLIGHT_COLOUR: olc::Pixel = olc::Pixel {
    r: 255,
    g: 200,
    b: 100,
    a: 255,
};

pub const POINT_COLOUR: olc::Pixel =
{
    olc::Pixel {
        r: 240,
        g: 150,
        b: 50,
        a: 255,
    }
};

pub const SYMBOL_WIDTH: usize = 9;
pub const SYMBOL_HEIGHT: usize = 12;
/*
____#____
__#_#_#__
__#_#_#_#
__#_#_#_#
__#_#_#_#
#_#_#_#_#
#_#######
#########
_########
_########
__######_
__######_
*/

pub const HAND_MODE_SYMBOL: [bool; SYMBOL_HEIGHT * SYMBOL_WIDTH] = [
    false, false, false, false, true, false, false, false, false, false, false, true, false, true,
    false, true, false, false, false, false, true, false, true, false, true, false, true, false,
    false, true, false, true, false, true, false, true, false, false, true, false, true, false,
    true, false, true, true, false, true, false, true, false, true, false, true, true, false, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    false, true, true, true, true, true, true, true, true, false, true, true, true, true, true,
    true, true, true, false, false, true, true, true, true, true, true, false, false, false, true,
    true, true, true, true, true, false,
];

/*
-#-----#-
-#-----#-
--#---#--
--#---#--
---#-#---
---#-#---
----#----
---#-#---
---#-#---
###---###
#-#---#-#
###---###
*/

pub const CUT_MODE_SYMBOL: [bool; SYMBOL_HEIGHT * SYMBOL_WIDTH] = [
    false, true, false, false, false, false, false, true, false, false, true, false, false, false,
    false, false, true, false, false, false, true, false, false, false, true, false, false, false,
    false, true, false, false, false, true, false, false, false, false, false, true, false, true,
    false, false, false, false, false, false, true, false, true, false, false, false, false, false,
    false, false, true, false, false, false, false, false, false, false, true, false, true, false,
    false, false, false, false, false, true, false, true, false, false, false, true, true, true,
    false, false, false, true, true, true, true, false, true, false, false, false, true, false,
    true, true, true, true, false, false, false, true, true, true,
];

/*
---------
----#----
-#--#--#-
---------
---###---
##-###-##
---###---
---------
-#--#--#-
----#----
---------
---------
*/

pub const PLACE_MODE_SYMBOL: [bool; SYMBOL_HEIGHT * SYMBOL_WIDTH] = [
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    true, false, false, false, false, false, true, false, false, true, false, false, true, false,
    false, false, false, false, false, false, false, false, false, false, false, false, true, true,
    true, false, false, false, true, true, false, true, true, true, false, true, true, false,
    false, false, true, true, true, false, false, false, false, false, false, false, false, false,
    false, false, false, false, true, false, false, true, false, false, true, false, false, false,
    false, false, true, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false,
];

/*
---------
---------
--###--##
--###----
--###--##
------#--
--#--#-#-
--#--#---
---------
---------
---------
---------
*/

pub const FORCE_MODE_SYMBOL: [bool; SYMBOL_HEIGHT * SYMBOL_WIDTH] = [
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, true, true, true, false, false, true, true,
    false, false, true, true, true, false, false, false, false, false, false, true, true, true,
    false, false, true, true, false, false, false, false, false, false, true, false, false, false,
    false, true, false, false, true, false, true, false, false, false, true, false, false, true,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
];
