use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use image::{DynamicImage, GenericImage, GenericImageView};
use imager_av::classifier::{self, Class};
use imager_av::vmaf;
use crate::encode::lossy::{encode};

#[derive(Clone, Serialize, Deserialize)]
pub struct OutMeta {
    pub class: Class,
    pub score: f64,
    pub end_q: u32,
    pub passed: bool,
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
}

pub fn opt(source: &DynamicImage) -> (Vec<u8>, OutMeta) {
    let class = classifier::report(source);
    let vmaf_source = vmaf::Yuv420pImage::from_image(source);
    let run = |q: f32| -> (Vec<u8>, f64) {
        let compressed = encode(source, q);
        let score = {
            let vmaf_derivative = crate::decode::decode(&compressed);
            let vmaf_derivative = vmaf::Yuv420pImage::from_image(&vmaf_derivative);
            vmaf::report(&vmaf_source, &vmaf_derivative)
        };
        (compressed, score)
    };
    let fallback = |end_q, score| {
        let compressed = encode(source, 100.0);
        let meta = OutMeta {
            class: class.class.clone(),
            score,
            end_q,
            passed: false,
            input_path: None,
            output_path: None,
        };
        (compressed, meta)
    };
    let terminate = |score: f64| {
        let (width, height) = source.dimensions();
        let is_small = {
            width < 600 || height < 600
        };
        let mut threshold;
        match class.class {
            Class::L0 | Class::L1 | Class::L2 if is_small => {
                threshold = 99.0;
            }
            Class::L0 | Class::L1 | Class::L2 => {
                threshold = 95.0;
            }
            Class::M1 => {
                if is_small {
                    threshold = 98.0;
                } else {
                    threshold = 90.0;
                }
            }
            Class::H1 | Class::H2 if is_small => {
                threshold = 88.0;
            }
            Class::H1 => {
                threshold = 75.0;
            }
            Class::H2 => {
                threshold = 65.0;
            }
        }
        score >= threshold
    };
    // SEARCH
    let start_q = match class.class {
        Class::H1 | Class::H2 => 1,
        _ => 10,
    };
    let mut last_q = None;
    let mut last_score = None;
    for q in start_q..100 {
        let (compressed, score) = run(q as f32);
        last_q = Some(q);
        last_score = Some(score);
        if terminate(score) {
            let meta = OutMeta {
                class: class.class.clone(),
                score,
                end_q: q,
                passed: true,
                input_path: None,
                output_path: None,
            };
            return (compressed, meta);
        }
    }
    // FALLBACK
    let last_q = last_q.expect("should run at least once");
    let last_score = last_score.expect("should run at least once");
    fallback(last_q, last_score)
}


///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub(crate) fn process(from: &str) -> Option<OutMeta> {
    // SETUP
    let path = PathBuf::from(from);
    let file_name = path
        .file_name()
        .expect("source filename");
    // RUN
    let source = ::image::open(from).expect("read source image");
    let (output, mut meta) = opt(&source);
    // SETUP OUTPUT
    let mut output_path = PathBuf::from("assets/output/");
    output_path.push(format!("{}", meta.class));
    std::fs::create_dir_all(&output_path).expect("create parent dir");
    output_path.push(file_name);
    output_path.set_extension("webp");
    // LOG INFO
    let mutex = std::sync::Mutex::new(());
    let lock = mutex.lock().expect("mutex lock");
    if meta.passed {
        println!("[ {} ] :", output_path.to_str().expect("PathBuf to str"));
        println!("\tclass : {:?}", meta.class);
        println!("\tscore : {}", meta.score);
        println!("\t    q : {}", meta.end_q);
    } else {
        eprintln!("[FAILED] [ {} ] :", output_path.to_str().expect("PathBuf to str"));
        eprintln!("\tclass : {:?}", meta.class);
        eprintln!("\tscore : {}", meta.score);
        eprintln!("\t    q : {}", meta.end_q);
    }
    std::mem::drop(lock);
    std::fs::write(&output_path, output).expect("save file");
    // DONE
    meta.input_path = Some(path);
    meta.output_path = Some(output_path);
    Some(meta)
}

pub(crate) fn run() {
    let paths = &[
        "assets/samples/big/high/2yR-nZN4yo4MAq.jpeg",
        "assets/samples/big/high/4qp-MMjpZ5YrpW.jpeg",
        "assets/samples/big/high/71w-X9l4nR9QLY.jpeg",
        "assets/samples/big/high/8DL-N4x9jnZM0N.jpeg",
        "assets/samples/big/high/9m8-xYdJ495eE8.jpeg",
        "assets/samples/big/high/9nj-qPYoBgPyAw.jpeg",
        "assets/samples/big/high/Bzm-ppOwKWeogb.jpeg",
        "assets/samples/big/high/Dow-0xbYR1PMDY.jpeg",
        "assets/samples/big/high/Gzj-DyjXR7ZWXO.jpeg",
        "assets/samples/big/high/NLw-zDer7Oq9BB.jpeg",
        "assets/samples/big/high/NOq-rwA5NEgLbM.jpeg",
        "assets/samples/big/high/PE3-wywb3Z1nMj.jpeg",
        "assets/samples/big/high/V3X-0dVBQxPDBj.jpeg",
        "assets/samples/big/high/brN-JwleqW0AZz.jpeg",
        "assets/samples/big/high/by2-YbbGlY7V0m.jpeg",
        "assets/samples/big/high/d4y-2OZqp8KpP2.jpeg",
        "assets/samples/big/high/eEz-g9YQdR7bE3.jpeg",
        "assets/samples/big/high/eQm-4qr9PKzdWe.jpeg",
        "assets/samples/big/high/j3j-Xj4b1RJXNE.jpeg",
        "assets/samples/big/high/lJV-ZDbAN2Pr3p.jpeg",
        "assets/samples/big/high/nPP-dg2PyLrPgR.jpeg",
        "assets/samples/big/high/nPX-NAOMqDx3M.jpeg",
        "assets/samples/big/high/x9G-5Vz295797J.jpeg",
        "assets/samples/big/high/zn9-3XgzBmox8.jpeg",
        "assets/samples/big/low/0KR-nQjoAQZQQY.jpeg",
        "assets/samples/big/low/1Ye-Qmb0MyqXjA.jpeg",
        "assets/samples/big/low/2NM-NVeXK43wom.jpeg",
        "assets/samples/big/low/2gp-PxOQEEMyqd.jpeg",
        "assets/samples/big/low/2yV-pyOxnPw300.jpeg",
        "assets/samples/big/low/3Zm-Yb17g7VxYY.jpeg",
        "assets/samples/big/low/4KD-AQypZWJGdQ.jpeg",
        "assets/samples/big/low/5D5-2omBX9JdVr.jpeg",
        "assets/samples/big/low/7BR-oNAbXLN95x.jpeg",
        "assets/samples/big/low/7Jm-2BnJK8qJ2G.jpeg",
        "assets/samples/big/low/7XR-LE7Emxdlx.jpeg",
        "assets/samples/big/low/7dD-AG2z3poDmN.jpeg",
        "assets/samples/big/low/7jd-dDJjK50gKG.jpeg",
        "assets/samples/big/low/8XG-xPqxP0jmJQ.jpeg",
        "assets/samples/big/low/8do-yPNK4VxpgX.jpeg",
        "assets/samples/big/low/AJG-Az2xDl2ogp.jpeg",
        "assets/samples/big/low/AVD-9bDVMQL0ER.jpeg",
        "assets/samples/big/low/AWG-2wxD40XpxE.jpeg",
        "assets/samples/big/low/AWP-RxoyBppnJr.jpeg",
        "assets/samples/big/low/BLW-PK9z1QlQJw.jpeg",
        "assets/samples/big/low/Dbl-jxQeq7L0q5.jpeg",
        "assets/samples/big/low/Dbx-LNRB2LZE7y.jpeg",
        "assets/samples/big/low/DdN-ooG5bXOp37.jpeg",
        "assets/samples/big/low/E1g-5yEqjy1bez.jpeg",
        "assets/samples/big/low/EJR-lolXzJyeXn.jpeg",
        "assets/samples/big/low/EWP-m3XrrEyzzz.jpeg",
        "assets/samples/big/low/EqN-rEORAybW0e.jpeg",
        "assets/samples/big/low/G2j-ZKQ5mYO5bO.jpeg",
        "assets/samples/big/low/GLY-wwOMGx7APo.jpeg",
        "assets/samples/big/low/J3P-5PrqVx1xd4.jpeg",
        "assets/samples/big/low/Jye-qDoo8DWA0V.jpeg",
        "assets/samples/big/low/K8V-z822egwgVV.jpeg",
        "assets/samples/big/low/KNW-DXpwJpmb53.jpeg",
        "assets/samples/big/low/KnN-4nq3D3Wplj.jpeg",
        "assets/samples/big/low/M4A-bn5QqmoDMe.jpeg",
        "assets/samples/big/low/MB4-R7K0w3LVYW.jpeg",
        "assets/samples/big/low/MZ0-OmPZQq07o4.jpeg",
        "assets/samples/big/low/N2y-LAwMnb53ZG.jpeg",
        "assets/samples/big/low/NbV-ZGy4B31xB1.jpeg",
        "assets/samples/big/low/Ojp-M7NJm3m3Mp.jpeg",
        "assets/samples/big/low/OoJ-R24O4lPYJ8.jpeg",
        "assets/samples/big/low/OoQ-9p7oedM9Dw.jpeg",
        "assets/samples/big/low/PXZ-Z33LyezdYx.jpeg",
        "assets/samples/big/low/PZB-qyBGVexN9n.jpeg",
        "assets/samples/big/low/QKJ-5n2Wpzy38g.jpeg",
        "assets/samples/big/low/RLz-353EwpwmNz.jpeg",
        "assets/samples/big/low/V5Q-z30V2MzjBj.jpeg",
        "assets/samples/big/low/Vyo-LGMJQm9RRm.jpeg",
        "assets/samples/big/low/WyR-ZgYG94g5B.jpeg",
        "assets/samples/big/low/YqD-qe0Gzz1GjZ.jpeg",
        "assets/samples/big/low/b34-4NWN87JwKg.jpeg",
        "assets/samples/big/low/b9B-ngOzZ3yMVQ.jpeg",
        "assets/samples/big/low/bxM-qD9wOXNJMn.jpeg",
        "assets/samples/big/low/bzK-3GZY9D1X0Q.jpeg",
        "assets/samples/big/low/eP8-gM3JorZBgp.jpeg",
        "assets/samples/big/low/eQN-zR4rErlrOO.jpeg",
        "assets/samples/big/low/gjn-4dR7xqo8YK.jpeg",
        "assets/samples/big/low/jAe-jwWz4AwZ32.jpeg",
        "assets/samples/big/low/lLl-BVo8Znro8b.jpeg",
        "assets/samples/big/low/loO-q1ZgV8mNBo.jpeg",
        "assets/samples/big/low/mrK-rx5jPKPm0Q.jpeg",
        "assets/samples/big/low/n3l-bg7wbNKjzM.jpeg",
        "assets/samples/big/low/noj-42qXpE5KN8.jpeg",
        "assets/samples/big/low/nxl-B02dnXwe7W.jpeg",
        "assets/samples/big/low/o3w-Xb0jZjBy2x.jpeg",
        "assets/samples/big/low/oLX-XAl7JZOAmL.jpeg",
        "assets/samples/big/low/oZx-EezNPyyW7L.jpeg",
        "assets/samples/big/low/ooQ-4rWBWr4XPZ.jpeg",
        "assets/samples/big/low/pGX-E11WJ92qEX.jpeg",
        "assets/samples/big/low/pM0-NmjR24BAwo.jpeg",
        "assets/samples/big/low/pZQ-eQLEKEM3B.jpeg",
        "assets/samples/big/low/q9E-Kd4YXA2WY2.jpeg",
        "assets/samples/big/low/qDW-dmq1exJwQj.jpeg",
        "assets/samples/big/low/qZE-Xd71135GPj.jpeg",
        "assets/samples/big/low/wWr-l0jLqeBJBB.jpeg",
        "assets/samples/big/low/wxY-JnRng0yWwA.jpeg",
        "assets/samples/big/low/xOw-2BlrR97jDN.jpeg",
        "assets/samples/big/low/xdb-4g9lbP8jD8.jpeg",
        "assets/samples/big/low/zGV-WlMEVAP54O.jpeg",
    ];

    let results = paths
        .par_iter()
        .filter_map(|s| process(s))
        .collect::<Vec<_>>();
    let text = serde_json::to_string_pretty(&results).expect("log meta as json");
    std::fs::write("assets/output/data.json", text);
}