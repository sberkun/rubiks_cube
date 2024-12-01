use std::{str::FromStr, time::Instant};



type Color = u8;
const WHITE: Color = 0;
const GREEN: Color = 1;
const RED: Color = 2;
const YELLOW: Color = 3;
const BLUE: Color = 4;
const ORANGE: Color = 5;

const TL: u8 = 0;
const TM: u8 = 1;
const TR: u8 = 2;
const CR: u8 = 3;
const BR: u8 = 4;
const BM: u8 = 5;
const BL: u8 = 6;
const CL: u8 = 7;
const CM: u8 = 0xFF;

const IDENTITY: Cube = Cube([0x00000000, 0x11111111, 0x22222222, 0x33333333, 0x44444444, 0x55555555]);

fn color_to_string(color: Color) -> &'static str {
    match color {
        WHITE => "white",
        GREEN => "green",
        RED => "red",
        YELLOW => "yellow",
        BLUE => "blue",
        ORANGE => "orange",
        _ => "???"
    }
}

fn op_to_string(op: &str, color: Color, append: String) -> String {
    let mut cheese = String::new();
    cheese.push_str(op);
    cheese.push_str(color_to_string(color));
    cheese.push_str(" ");
    cheese.push_str(&append);
    cheese
}

fn char_to_color(chr: u8) -> Color {
    match chr {
        b'w' => WHITE,
        b'o' => ORANGE,
        b'g' => GREEN,
        b'r' => RED,
        b'b' => BLUE,
        b'y' => YELLOW,
        _ => panic!("bad")
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
struct Cube([u32; 6]);

impl Cube {
    fn parse_str(strs: [&str;18]) -> Cube {
        let colors = [
                    WHITE,              // <
                    WHITE,
                    WHITE,
            ORANGE, GREEN, RED, BLUE, // < ^ < v 
            ORANGE, GREEN, RED, BLUE,
            ORANGE, GREEN, RED, BLUE,
                    YELLOW,              // >
                    YELLOW,
                    YELLOW,
        ];
        let indices = [
                          [TR, CR, BR],
                          [TM, CM, BM],
                          [TL, CL, BL],
            [TR, CR, BR], [TL, TM, TR], [TR, CR, BR], [BR, BM, BL],
            [TM, CM, BM], [CL, CM, CR], [TM, CM, BM], [CR, CM, CL],
            [TL, CL, BL], [BL, BM, BR], [TL, CL, BL], [TR, TM, TL],
                          [BL, CL, TL],
                          [BM, CM, TM],
                          [BR, CR, TR],
        ];

        let mut ret = Cube([0,0,0,0,0,0]);
        for s in 0..strs.len() {
            let chrs = strs[s].as_bytes();
            let inds = indices[s];
            for a in 0..3 {
                if inds[a] != CM {
                    ret.set_square(colors[s], inds[a], char_to_color(chrs[a]));
                } else {
                    assert!(char_to_color(chrs[a]) == colors[s])
                }
            }
        }

        ret
    }


    #[inline]
    fn get_square(&self, face: Color, index: u8) -> Color {
        let face: u32 = self.0[face as usize];
        ((face >> (index * 4)) & 7) as Color
    }

    #[inline]
    fn set_square(&mut self, face: Color, index: u8, value: Color) {
        let old_face: u32 = self.0[face as usize];
        let old_face_clear = old_face & !(7 << (index * 4));
        let new_face = old_face_clear | ((value as u32) << (index * 4));
        self.0[face as usize] = new_face;
    }

    fn rotate_across_faces(&mut self, base_face: Color, color_offsets: (u8, u8, u8, u8), index: (u8, u8, u8, u8)) {
        let faces: (u8, u8, u8, u8) = ((base_face + color_offsets.0) % 6,
                                       (base_face + color_offsets.1) % 6,
                                       (base_face + color_offsets.2) % 6,
                                       (base_face + color_offsets.3) % 6);
        let olds: (u8, u8, u8, u8) = (self.get_square(faces.0, index.0),
                                      self.get_square(faces.1, index.1),
                                      self.get_square(faces.2, index.2),
                                      self.get_square(faces.3, index.3));
        self.set_square(faces.0, index.0, olds.3);
        self.set_square(faces.1, index.1, olds.0);
        self.set_square(faces.2, index.2, olds.1);
        self.set_square(faces.3, index.3, olds.2);
    }

    fn rotate_within_face(&mut self, face: Color, index: (u8, u8, u8, u8)) {
        self.rotate_across_faces(face, (0,0,0,0), index);
    }

    fn rotate_clockwise(&mut self, face: Color) {
        // white goes clockwise
        // green top -> orange right -> blue bot -> red right -> green top
        // +1              +5            +4          +2           +1

        // yellow goes clockwise
        // blue top -> orange left -> green bot -> red left -> blue top
        // +1           +2             +4           +5          +1

        self.rotate_within_face(face, (TL, TR, BR, BL));
        self.rotate_within_face(face, (TM, CR, BM, CL));
        if face % 2 == 0 {
            self.rotate_across_faces(face, (1,5,4,2), (TL, TR, BR, TR));
            self.rotate_across_faces(face, (1,5,4,2), (TM, CR, BM, CR));
            self.rotate_across_faces(face, (1,5,4,2), (TR, BR, BL, BR));
        } else {
            self.rotate_across_faces(face, (1,2,4,5), (TL, BL, BR, BL));
            self.rotate_across_faces(face, (1,2,4,5), (TM, CL, BM, CL));
            self.rotate_across_faces(face, (1,2,4,5), (TR, TL, BL, TL));
        }
    }

    fn rotate_counterclockwise(&mut self, face: Color) {
        // white goes counterclockwise
        // green top -> red right -> blue bot -> orange right -> green top
        // +1            +2          +4            +5            +1

        // yellow goes counterclockwise
        // blue top -> red left -> green bot -> orange left -> blue top
        // +1           +5          +4            +2            +1

        self.rotate_within_face(face, (TR, TL, BL, BR));
        self.rotate_within_face(face, (TM, CL, BM, CR));
        if face % 2 == 0 {
            self.rotate_across_faces(face, (1,2,4,5), (TL, TR, BR, TR));
            self.rotate_across_faces(face, (1,2,4,5), (TM, CR, BM, CR));
            self.rotate_across_faces(face, (1,2,4,5), (TR, BR, BL, BR));
        } else {
            self.rotate_across_faces(face, (1,5,4,2), (TL, BL, BR, BL));
            self.rotate_across_faces(face, (1,5,4,2), (TM, CL, BM, CL));
            self.rotate_across_faces(face, (1,5,4,2), (TR, TL, BL, TL));
        }
    }

    fn rotate_180(&mut self, face: Color) {
        self.rotate_clockwise(face);
        self.rotate_clockwise(face);
    }

    fn better_rotate<const face: u8>(&mut self) {
        self.rotate_clockwise(face);
    }
}


fn match_corner(cube: &Cube, color1: Color, color2: Color, color3: Color,
              index1: u8, index2: u8, index3: u8,
              value1: Color, value2: Color, value3: Color) -> bool {
    
    (1 << cube.get_square(color1, index1)) |
    (1 << cube.get_square(color2, index2)) |
    (1 << cube.get_square(color3, index3))
    ==
    (1 << value1) | (1 << value2) | (1 << value3)
}

fn ending_condition(cube: &Cube) -> bool {
    // w g r y b o
    // Cube([00000000, 03330000, 33000003, 33333333, 00000333, 33000003])
    let gmask = 0x07770000;
    let rmask = 0x00077700;
    cube.0[YELLOW as usize] == IDENTITY.0[YELLOW as usize]
    && cube.0[GREEN as usize] & gmask == IDENTITY.0[GREEN as usize] & gmask 

    && match_corner(cube, GREEN, WHITE, ORANGE, TL, TL, BR, RED, GREEN, WHITE)
    && match_corner(cube, GREEN, WHITE, RED, TR, BL, TR, BLUE, RED, WHITE)
    && match_corner(cube, RED, BLUE, WHITE, BR, BR, BR, ORANGE, GREEN, WHITE)
}

fn find_solution(cube: &Cube, desired_depth: usize) -> Option<String> {
    if ending_condition(cube) {
        return Some(String::new())
    }
    if desired_depth == 0 {
        return None
    }

    if let Some(sol) = check_rotation::<1>(cube, desired_depth - 1) {
        return Some(sol);
    }
    if let Some(sol) = check_rotation::<4>(cube, desired_depth - 1) {
        return Some(sol);
    }
    if let Some(sol) = check_rotation::<2>(cube, desired_depth - 1) {
        return Some(sol);
    }
    if let Some(sol) = check_rotation::<5>(cube, desired_depth - 1) {
        return Some(sol);
    }
    if let Some(sol) = check_rotation::<0>(cube, desired_depth - 1) {
        return Some(sol);
    }
    if let Some(sol) = check_rotation::<3>(cube, desired_depth - 1) {
        return Some(sol);
    }

    None
}


fn check_rotation<const face: u8>(cube: &Cube, desired_depth: usize) -> Option<String> {
    let mut new_cube = cube.clone();
    new_cube.better_rotate::<face>();
    if let Some(sol) = find_solution(&new_cube, desired_depth) {
        return Some(op_to_string("-", face, sol));
    }
    new_cube.better_rotate::<face>();
    if let Some(sol) = find_solution(&new_cube, desired_depth) {
        return Some(op_to_string("=", face, sol));
    }
    new_cube.better_rotate::<face>();
    if let Some(sol) = find_solution(&new_cube, desired_depth) {
        return Some(op_to_string("+", face, sol));
    }
    None
}




fn main() {
    let cheese = Cube::parse_str([
            "www",
            "www",
            "www",
        "www","yww","www","www",
        "wow","wgw","wrw","wbw",
        "yyy","yyy","yyy","yyy",
            "yyy",
            "yyy",
            "yyy",
    ]);
    println!("{:08x?}", cheese);
    let mut idencopy = IDENTITY.clone();
    idencopy.rotate_clockwise(WHITE);
    println!("{}", idencopy == cheese);
    let tt= Instant::now();
    println!("{:?}", find_solution(&IDENTITY, 8));
    println!("{:?}", tt.elapsed());
}
