//! Lookup table to easily perform operations on correlation values
//! This code has been directly copied over from Hxro Network Dexterity codebase

// use crate::fast_int::FastInt;
use spicenet_shared::fast_int::FastInt;

pub const CORRELATION_LOOKUP_TABLE: [FastInt; 256] = [
    FastInt {
        value: -1000000_i128,
    },
    FastInt {
        value: -992187_i128,
    },
    FastInt {
        value: -984375_i128,
    },
    FastInt {
        value: -976562_i128,
    },
    FastInt {
        value: -968750_i128,
    },
    FastInt {
        value: -960937_i128,
    },
    FastInt {
        value: -953125_i128,
    },
    FastInt {
        value: -945312_i128,
    },
    FastInt {
        value: -937500_i128,
    },
    FastInt {
        value: -929687_i128,
    },
    FastInt {
        value: -921875_i128,
    },
    FastInt {
        value: -914062_i128,
    },
    FastInt {
        value: -906250_i128,
    },
    FastInt {
        value: -898437_i128,
    },
    FastInt {
        value: -890625_i128,
    },
    FastInt {
        value: -882812_i128,
    },
    FastInt {
        value: -875000_i128,
    },
    FastInt {
        value: -867187_i128,
    },
    FastInt {
        value: -859375_i128,
    },
    FastInt {
        value: -851562_i128,
    },
    FastInt {
        value: -843750_i128,
    },
    FastInt {
        value: -835937_i128,
    },
    FastInt {
        value: -828125_i128,
    },
    FastInt {
        value: -820312_i128,
    },
    FastInt {
        value: -812500_i128,
    },
    FastInt {
        value: -804687_i128,
    },
    FastInt {
        value: -796875_i128,
    },
    FastInt {
        value: -789062_i128,
    },
    FastInt {
        value: -781250_i128,
    },
    FastInt {
        value: -773437_i128,
    },
    FastInt {
        value: -765625_i128,
    },
    FastInt {
        value: -757812_i128,
    },
    FastInt {
        value: -750000_i128,
    },
    FastInt {
        value: -742187_i128,
    },
    FastInt {
        value: -734375_i128,
    },
    FastInt {
        value: -726562_i128,
    },
    FastInt {
        value: -718750_i128,
    },
    FastInt {
        value: -710937_i128,
    },
    FastInt {
        value: -703125_i128,
    },
    FastInt {
        value: -695312_i128,
    },
    FastInt {
        value: -687500_i128,
    },
    FastInt {
        value: -679687_i128,
    },
    FastInt {
        value: -671875_i128,
    },
    FastInt {
        value: -664062_i128,
    },
    FastInt {
        value: -656250_i128,
    },
    FastInt {
        value: -648437_i128,
    },
    FastInt {
        value: -640625_i128,
    },
    FastInt {
        value: -632812_i128,
    },
    FastInt {
        value: -625000_i128,
    },
    FastInt {
        value: -617187_i128,
    },
    FastInt {
        value: -609375_i128,
    },
    FastInt {
        value: -601562_i128,
    },
    FastInt {
        value: -593750_i128,
    },
    FastInt {
        value: -585937_i128,
    },
    FastInt {
        value: -578125_i128,
    },
    FastInt {
        value: -570312_i128,
    },
    FastInt {
        value: -562500_i128,
    },
    FastInt {
        value: -554687_i128,
    },
    FastInt {
        value: -546875_i128,
    },
    FastInt {
        value: -539062_i128,
    },
    FastInt {
        value: -531250_i128,
    },
    FastInt {
        value: -523437_i128,
    },
    FastInt {
        value: -515625_i128,
    },
    FastInt {
        value: -507812_i128,
    },
    FastInt {
        value: -500000_i128,
    },
    FastInt {
        value: -492187_i128,
    },
    FastInt {
        value: -484375_i128,
    },
    FastInt {
        value: -476562_i128,
    },
    FastInt {
        value: -468750_i128,
    },
    FastInt {
        value: -460937_i128,
    },
    FastInt {
        value: -453125_i128,
    },
    FastInt {
        value: -445312_i128,
    },
    FastInt {
        value: -437500_i128,
    },
    FastInt {
        value: -429687_i128,
    },
    FastInt {
        value: -421875_i128,
    },
    FastInt {
        value: -414062_i128,
    },
    FastInt {
        value: -406250_i128,
    },
    FastInt {
        value: -398437_i128,
    },
    FastInt {
        value: -390625_i128,
    },
    FastInt {
        value: -382812_i128,
    },
    FastInt {
        value: -375000_i128,
    },
    FastInt {
        value: -367187_i128,
    },
    FastInt {
        value: -359375_i128,
    },
    FastInt {
        value: -351562_i128,
    },
    FastInt {
        value: -343750_i128,
    },
    FastInt {
        value: -335937_i128,
    },
    FastInt {
        value: -328125_i128,
    },
    FastInt {
        value: -320312_i128,
    },
    FastInt {
        value: -312500_i128,
    },
    FastInt {
        value: -304687_i128,
    },
    FastInt {
        value: -296875_i128,
    },
    FastInt {
        value: -289062_i128,
    },
    FastInt {
        value: -281250_i128,
    },
    FastInt {
        value: -273437_i128,
    },
    FastInt {
        value: -265625_i128,
    },
    FastInt {
        value: -257812_i128,
    },
    FastInt {
        value: -250000_i128,
    },
    FastInt {
        value: -242187_i128,
    },
    FastInt {
        value: -234375_i128,
    },
    FastInt {
        value: -226562_i128,
    },
    FastInt {
        value: -218750_i128,
    },
    FastInt {
        value: -210937_i128,
    },
    FastInt {
        value: -203125_i128,
    },
    FastInt {
        value: -195312_i128,
    },
    FastInt {
        value: -187500_i128,
    },
    FastInt {
        value: -179687_i128,
    },
    FastInt {
        value: -171875_i128,
    },
    FastInt {
        value: -164062_i128,
    },
    FastInt {
        value: -156250_i128,
    },
    FastInt {
        value: -148437_i128,
    },
    FastInt {
        value: -140625_i128,
    },
    FastInt {
        value: -132812_i128,
    },
    FastInt {
        value: -125000_i128,
    },
    FastInt {
        value: -117187_i128,
    },
    FastInt {
        value: -109375_i128,
    },
    FastInt {
        value: -101562_i128,
    },
    FastInt { value: -93750_i128 },
    FastInt { value: -85937_i128 },
    FastInt { value: -78125_i128 },
    FastInt { value: -70312_i128 },
    FastInt { value: -62500_i128 },
    FastInt { value: -54687_i128 },
    FastInt { value: -46875_i128 },
    FastInt { value: -39062_i128 },
    FastInt { value: -31250_i128 },
    FastInt { value: -23437_i128 },
    FastInt { value: -15625_i128 },
    FastInt { value: -7812_i128 },
    FastInt { value: 0_i128 },
    FastInt { value: 7812_i128 },
    FastInt { value: 15625_i128 },
    FastInt { value: 23437_i128 },
    FastInt { value: 31250_i128 },
    FastInt { value: 39062_i128 },
    FastInt { value: 46875_i128 },
    FastInt { value: 54687_i128 },
    FastInt { value: 62500_i128 },
    FastInt { value: 70312_i128 },
    FastInt { value: 78125_i128 },
    FastInt { value: 85937_i128 },
    FastInt { value: 93750_i128 },
    FastInt { value: 101562_i128 },
    FastInt { value: 109375_i128 },
    FastInt { value: 117187_i128 },
    FastInt { value: 125000_i128 },
    FastInt { value: 132812_i128 },
    FastInt { value: 140625_i128 },
    FastInt { value: 148437_i128 },
    FastInt { value: 156250_i128 },
    FastInt { value: 164062_i128 },
    FastInt { value: 171875_i128 },
    FastInt { value: 179687_i128 },
    FastInt { value: 187500_i128 },
    FastInt { value: 195312_i128 },
    FastInt { value: 203125_i128 },
    FastInt { value: 210937_i128 },
    FastInt { value: 218750_i128 },
    FastInt { value: 226562_i128 },
    FastInt { value: 234375_i128 },
    FastInt { value: 242187_i128 },
    FastInt { value: 250000_i128 },
    FastInt { value: 257812_i128 },
    FastInt { value: 265625_i128 },
    FastInt { value: 273437_i128 },
    FastInt { value: 281250_i128 },
    FastInt { value: 289062_i128 },
    FastInt { value: 296875_i128 },
    FastInt { value: 304687_i128 },
    FastInt { value: 312500_i128 },
    FastInt { value: 320312_i128 },
    FastInt { value: 328125_i128 },
    FastInt { value: 335937_i128 },
    FastInt { value: 343750_i128 },
    FastInt { value: 351562_i128 },
    FastInt { value: 359375_i128 },
    FastInt { value: 367187_i128 },
    FastInt { value: 375000_i128 },
    FastInt { value: 382812_i128 },
    FastInt { value: 390625_i128 },
    FastInt { value: 398437_i128 },
    FastInt { value: 406250_i128 },
    FastInt { value: 414062_i128 },
    FastInt { value: 421875_i128 },
    FastInt { value: 429687_i128 },
    FastInt { value: 437500_i128 },
    FastInt { value: 445312_i128 },
    FastInt { value: 453125_i128 },
    FastInt { value: 460937_i128 },
    FastInt { value: 468750_i128 },
    FastInt { value: 476562_i128 },
    FastInt { value: 484375_i128 },
    FastInt { value: 492187_i128 },
    FastInt { value: 500000_i128 },
    FastInt { value: 507812_i128 },
    FastInt { value: 515625_i128 },
    FastInt { value: 523437_i128 },
    FastInt { value: 531250_i128 },
    FastInt { value: 539062_i128 },
    FastInt { value: 546875_i128 },
    FastInt { value: 554687_i128 },
    FastInt { value: 562500_i128 },
    FastInt { value: 570312_i128 },
    FastInt { value: 578125_i128 },
    FastInt { value: 585937_i128 },
    FastInt { value: 593750_i128 },
    FastInt { value: 601562_i128 },
    FastInt { value: 609375_i128 },
    FastInt { value: 617187_i128 },
    FastInt { value: 625000_i128 },
    FastInt { value: 632812_i128 },
    FastInt { value: 640625_i128 },
    FastInt { value: 648437_i128 },
    FastInt { value: 656250_i128 },
    FastInt { value: 664062_i128 },
    FastInt { value: 671875_i128 },
    FastInt { value: 679687_i128 },
    FastInt { value: 687500_i128 },
    FastInt { value: 695312_i128 },
    FastInt { value: 703125_i128 },
    FastInt { value: 710937_i128 },
    FastInt { value: 718750_i128 },
    FastInt { value: 726562_i128 },
    FastInt { value: 734375_i128 },
    FastInt { value: 742187_i128 },
    FastInt { value: 750000_i128 },
    FastInt { value: 757812_i128 },
    FastInt { value: 765625_i128 },
    FastInt { value: 773437_i128 },
    FastInt { value: 781250_i128 },
    FastInt { value: 789062_i128 },
    FastInt { value: 796875_i128 },
    FastInt { value: 804687_i128 },
    FastInt { value: 812500_i128 },
    FastInt { value: 820312_i128 },
    FastInt { value: 828125_i128 },
    FastInt { value: 835937_i128 },
    FastInt { value: 843750_i128 },
    FastInt { value: 851562_i128 },
    FastInt { value: 859375_i128 },
    FastInt { value: 867187_i128 },
    FastInt { value: 875000_i128 },
    FastInt { value: 882812_i128 },
    FastInt { value: 890625_i128 },
    FastInt { value: 898437_i128 },
    FastInt { value: 906250_i128 },
    FastInt { value: 914062_i128 },
    FastInt { value: 921875_i128 },
    FastInt { value: 929687_i128 },
    FastInt { value: 937500_i128 },
    FastInt { value: 945312_i128 },
    FastInt { value: 953125_i128 },
    FastInt { value: 960937_i128 },
    FastInt { value: 968750_i128 },
    FastInt { value: 976562_i128 },
    FastInt { value: 984375_i128 },
    FastInt { value: 992187_i128 },
];
