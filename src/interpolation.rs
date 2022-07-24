use std::cmp::Ord;

#[derive(Debug, Clone)]
pub struct Interpolator {
    x: Vec<f64>,
    y: Vec<f64>,
}

fn get_y_on_line(slope: f64, base: (f64, f64), x: f64) -> f64 {
    let (bx, by) = base;
    if bx == x {
      return by;
    }
    // by - y = (bx - x) * slope
    // y = by - (bx - x) * slope
    by - (bx - x) * slope
}

fn get_x_on_line(slope: f64, base: (f64, f64), y: f64) -> f64 {
    let (bx, by) = base;
    if by == y {
      return bx;
    }
    // by - y = (bx - x) * slope
    // x = bx - (by - y) / slope
    bx - (by - y) / slope
}

impl Interpolator
{
    fn get_slope_inner(&self, p1: usize, p2: usize) -> f64 {
        let p1_x = *self.x.get(p1).unwrap();
        let p1_y = *self.y.get(p1).unwrap();
        let p2_x = *self.x.get(p2).unwrap();
        let p2_y = *self.y.get(p2).unwrap();
        let res = (p1_y - p2_y) / (p1_x - p2_x);
        res
    }

    fn get_slope(&self, p: i64) -> f64 {
        if p <= 0 {
            self.get_slope_inner(0, 1)
        } else if p < self.x.len().try_into().unwrap() {
            self.get_slope_inner(p.try_into().unwrap(), (p - 1).try_into().unwrap())
        } else {
            self.get_slope_inner(self.x.len() - 2, self.x.len() - 1)
        }
    }

    pub fn with_points(mut points: Vec<(f64, f64)>) -> Self {
        assert!(points.len() >= 2, "points should contain >= 2 elements");
        points.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let (x, y) = points.into_iter().unzip();
        Self { x, y }
    }
    pub fn estimate_x(&self, y: f64) -> f64 {
        let pos = self.y.partition_point(|&item| item <= y);
        // 0..pos item <= y
        // pos..n item > y
        let slope = self.get_slope(pos as i64);
        let base = if pos == self.y.len() {
            (
                *self.x.last().unwrap(),
                *self.y.last().unwrap()
            )
        } else {
            (
                *self.x.get(pos).unwrap(),
                *self.y.get(pos).unwrap()
            )
        };
        get_x_on_line(slope, base, y)
    }
    pub fn estimate_y(&self, x: f64) -> f64  {
        let pos = self.x.partition_point(|&item| item <= x);
        // 0..pos item <= x
        // pos..n item > x
        let slope = self.get_slope(pos as i64);
        let base = if pos == self.x.len() {
            (
                *self.x.last().unwrap(),
                *self.y.last().unwrap()
            )
        } else {
            (
                *self.x.get(pos).unwrap(),
                *self.y.get(pos).unwrap()
            )
        };
        get_y_on_line(slope, base, x)
    }
}
