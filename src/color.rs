use std::ops::RangeInclusive;

pub struct Color{
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }
    }
}

impl Color{
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Result<Self, String>{
        let mut color = Self { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 };

        color.set_red(red)?;
        color.set_green(green)?;
        color.set_blue(blue)?;
        color.set_alpha(alpha)?;

        Ok(color)
    }

    pub fn new_random() -> Result<Self, String>{
        let mut color = Self { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 };

        color.set_random_color(0.0..=1.0,0.0..=1.0,0.0..=1.0,1.0..=1.0)?;

        Ok(color)
    }

    pub fn set_random_color(&mut self, range_red: RangeInclusive<f32>, range_green: RangeInclusive<f32>, range_blue: RangeInclusive<f32>, range_alpha: RangeInclusive<f32>) -> Result<(), String>{
        self.set_red(rand::random_range(range_red))?;
        self.set_green(rand::random_range(range_green))?;
        self.set_blue(rand::random_range(range_blue))?;
        self.set_alpha(rand::random_range(range_alpha))?;

        Ok(())
    }

    pub fn check_range(value: f32) -> Result<(), ()> {
        let is_invalid = |v: f32| v < 0.0 || v > 1.0;

        if is_invalid(value) {
            return Err(());
        }

        Ok(())
    }

    pub fn red(&self) -> f32 {
        self.red
    }

    pub fn green(&self) -> f32 {
        self.green
    }

    pub fn blue(&self) -> f32{
        self.blue
    }

    pub fn alpha(&self) -> f32{
        self.alpha
    }

    pub fn set_red(&mut self, red: f32) -> Result<(), String>{
        if Self::check_range(red).is_ok(){
            self.red = red;
        }else{
            return Err(format!("The set red is out of range: {}", red));
        }

        Ok(())
    }

    pub fn set_blue(&mut self,blue: f32) -> Result<(), String>{
        if Self::check_range(blue).is_ok(){
            self.blue = blue;
        }else{
            return Err(format!("The set blue is out of range: {}", blue));
        }

        Ok(())
    }

    pub fn set_green(&mut self, green: f32) -> Result<(), String>{
        if Self::check_range(green).is_ok(){
            self.green = green;
        }else{
            return Err(format!("The set green is out of range: {}", green));
        }

        Ok(())
    }

    pub fn set_alpha(&mut self,alpha: f32) -> Result<(), String>{
        if Self::check_range(alpha).is_ok(){
            self.alpha = alpha;
        }else{
            return Err(format!("The set alpha is out of range: {}", alpha));
        }

        Ok(())
    }
}
