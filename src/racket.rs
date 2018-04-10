#![feature(const_fn)]

use BGCOLOR;
use lcd;
use lcd::Framebuffer;
use lcd::FramebufferL8;
use graphics;

//general Racket Properties
const RACKET_WIDTH: u16 = 10;
const RACKET_HEIGHT: u16 = 30;
const RACKET_COLOR: lcd::Color = lcd::Color::rgb(150, 150, 30);

//Racket Positions
pub struct Racket {
    xpos_centre: u16,
    ypos_centre: u16,
    ypos_centre_old: u16,
}
impl Racket {
    //Create new Racket
    pub fn new(player_id: u8) -> Racket {
        if player_id == 0 {
            Racket {
                xpos_centre: RACKET_WIDTH,
                ypos_centre: 135,
                ypos_centre_old: 135,
            }
        } else {
            Racket {
                xpos_centre: 479 - RACKET_WIDTH,
                ypos_centre: 135,
                ypos_centre_old: 135,
            }
        }
    }
    //set Centre Point Coordinates
    pub fn set_ypos_centre(&mut self, ypos_centre_set: u16) {
        self.ypos_centre = ypos_centre_set;
    }
    pub fn set_ypos_centre_old(&mut self, ypos_centre_set: u16) {
        self.ypos_centre_old = ypos_centre_set;
    }

    //get Centre Point Coordinates
    pub fn get_xpos_centre(&self) -> u16 {
        self.xpos_centre
    }
    pub fn get_ypos_centre(&self) -> u16 {
        self.ypos_centre
    }
    pub fn get_ypos_centre_old(&self) -> u16 {
        self.ypos_centre_old
    }

    pub fn draw_racket(&self, buffer: &mut lcd::FramebufferL8) {
        graphics::draw_rectangle(
            buffer,
            self.xpos_centre - RACKET_WIDTH,
            self.xpos_centre + RACKET_WIDTH,
            self.ypos_centre - RACKET_HEIGHT,
            self.ypos_centre + RACKET_HEIGHT,
            RACKET_COLOR,
        );
    }
    pub fn draw_moved_racket(
        &self,
        buffer: &mut lcd::FramebufferL8,
        x_pos_centre: u16,

        y_top_erase: u16,
        y_bottom_erase: u16,
        y_top_draw: u16,
        y_bottom_draw: u16,
    ) {
        //erase old racket
        graphics::draw_rectangle(
            buffer,
            x_pos_centre - RACKET_WIDTH,
            x_pos_centre + RACKET_WIDTH,
            y_top_erase,
            y_bottom_erase,
            BGCOLOR,
        );
        //draw new racket
        graphics::draw_rectangle(
            buffer,
            x_pos_centre - RACKET_WIDTH,
            x_pos_centre + RACKET_WIDTH,
            y_top_draw,
            y_bottom_draw,
            RACKET_COLOR,
        );
    }
    //TODO Update racket from Server Gamestate
    /*pub fn update_racket_pos(&self, gamestate){
        
        //remember old position
        self.ypos_centre_old = self.ypos_centre;
        //TODO
        //update racket position
        self.ypos_centre =gamestate[1]}
*/
    /*pub fn update_racket_pos(&self, ){
    
        
            //if racket moved down
            if self.get_ypos_centre() > racket.get_ypos_centre_old() {
                racket.move_racket(
                    framebuffer,
                    racket.get_xpos_centre() - RACKET_WIDTH,
                    racket.get_xpos_centre() + RACKET_WIDTH,
                    racket.get_ypos_centre_old() - RACKET_HEIGHT,
                    min(
                        racket.get_ypos_centre() - RACKET_HEIGHT - 1,
                        racket.get_ypos_centre_old() + RACKET_HEIGHT,
                    ),
                    max(
                        racket.get_ypos_centre_old() + RACKET_HEIGHT,
                        racket.get_ypos_centre() - RACKET_HEIGHT,
                    ),
                    racket.get_ypos_centre() + RACKET_HEIGHT,
                    BGCOLOR,
                    RACKET_COLOR,
                );
            }
            //if racket moved up
            if racket.get_ypos_centre() < racket.get_ypos_centre_old() {
                //TODO CREATE FN MOVE RACKET
                racket.move_racket(
                    framebuffer,
                    racket.get_xpos_centre() - RACKET_WIDTH,
                    racket.get_xpos_centre() + RACKET_WIDTH,
                    max(
                        racket.get_ypos_centre() + RACKET_HEIGHT + 1,
                        racket.get_ypos_centre_old() - RACKET_HEIGHT,
                    ),
                    racket.get_ypos_centre_old() + RACKET_HEIGHT,
                    racket.get_ypos_centre() - RACKET_HEIGHT,
                    min(
                        racket.get_ypos_centre_old() - RACKET_HEIGHT,
                        racket.get_ypos_centre() + RACKET_HEIGHT,
                    ),
                    BGCOLOR,
                    RACKET_COLOR,
                );
            }
            //remember old racket points (y)
            let mut ypos_centre_old = racket.get_ypos_centre();
            racket.set_ypos_centre_old(ypos_centre_old);
        
    }*/
}
