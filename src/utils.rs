use std::collections::HashMap;

use anyhow::anyhow;
use ark_bn254::{Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use std::str::FromStr;

pub fn g1_to_string(value: G1Affine) -> [String; 3] {
    let Some(xy) = value.xy() else {
        return ["0".to_string(), "0".to_string(), "0".to_string()];
    };

    let x = xy.0.to_string();
    let y = xy.1.to_string();

    [x, y, "1".to_string()]
}

pub fn g2_to_string(value: G2Affine) -> [[String; 2]; 3] {
    let Some(xy) = value.xy() else {
        return [
            ["0".to_string(), "0".to_string()],
            ["0".to_string(), "0".to_string()],
            ["0".to_string(), "0".to_string()],
        ];
    };

    let x0 = xy.0.c0.to_string();
    let x1 = xy.0.c1.to_string();
    let y0 = xy.1.c0.to_string();
    let y1 = xy.1.c1.to_string();

    [[x0, x1], [y0, y1], ["1".to_string(), "0".to_string()]]
}

pub fn parse_g1(value: [String; 3]) -> Result<G1Affine, anyhow::Error> {
    let is_zero = value[2] == "0";
    if is_zero {
        return Ok(G1Affine::zero());
    }

    let x = Fq::from_str(&value[0]).map_err(|_| anyhow!("Failed to parse x coord"))?;
    let y = Fq::from_str(&value[1]).map_err(|_| anyhow!("Failed to parse y coord"))?;

    Ok(G1Affine::new_unchecked(x, y))
}

pub fn parse_f2(value: [[String; 2]; 3]) -> Result<G2Affine, anyhow::Error> {
    let is_zero = value[2][0] == "0" && value[2][1] == "0";
    if is_zero {
        return Ok(G2Affine::zero());
    }

    let x = Fq2::new(
        Fq::from_str(&value[0][0]).map_err(|_| anyhow!("Failed to parse x0 coord"))?,
        Fq::from_str(&value[0][1]).map_err(|_| anyhow!("Failed to parse x1 coord"))?,
    );
    let y = Fq2::new(
        Fq::from_str(&value[1][0]).map_err(|_| anyhow!("Failed to parse y0 coord"))?,
        Fq::from_str(&value[1][1]).map_err(|_| anyhow!("Failed to parse y1 coord"))?,
    );

    Ok(G2Affine::new_unchecked(x, y))
}

pub fn parse_pols(
    value: Vec<HashMap<u64, String>>,
) -> Result<Vec<HashMap<u64, Fr>>, anyhow::Error> {
    value
        .iter()
        .map(|map| {
            map.iter()
                .map(|(&k, v)| {
                    Ok((
                        k,
                        Fr::from_str(v).map_err(|_| anyhow!("Failed to parse Fr"))?,
                    ))
                })
                .collect()
        })
        .collect()
}
