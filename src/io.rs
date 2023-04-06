use std::{
    error::Error,
    fs::File,
    io::{ErrorKind, Read, Result as IOResult, Write},
};

use serde::{Deserialize, Serialize};

use crate::shamir_secret::Polynomial;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Header {
    length: u64,
    num_shares: usize,
    min_shares: usize,
    evaluation_point: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Body {
    data: Vec<u64>,
}

struct U64Iterator<T: Iterator<Item = IOResult<u8>>> {
    internal_iter: T,
    length: usize,
}

impl<T: Iterator<Item = IOResult<u8>>> U64Iterator<T> {
    pub fn new(internal: T) -> Self {
        U64Iterator {
            internal_iter: internal,
            length: 0,
        }
    }
}

impl<T: Iterator<Item = IOResult<u8>>> Iterator for U64Iterator<T> {
    type Item = IOResult<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bytes = [0u8; 8];
        match self.internal_iter.next()? {
            Ok(x) => {
                bytes[0] = x;
                self.length += 1;
                let mut err = Ok(0);
                for i in 1..8 {
                    match self.internal_iter.next().unwrap_or(Ok(0)) {
                        Ok(b) => {
                            bytes[i] = b;
                        }
                        Err(e) => {
                            err = Err(e);
                            break;
                        }
                    }
                }
                if err.is_err() {
                    Some(err)
                } else {
                    Some(Ok(u64::from_le_bytes(bytes)))
                }
            }

            Err(e) => {
                return Some(Err(e));
            }
        }
    }
}
pub fn encode(input: String, num_shares: usize, min_shares: usize) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input.clone())?;

    let input_length = input_file.metadata()?.len();

    let mut output_files = Vec::new();

    for i in 1..=num_shares {
        let mut file = File::create(format!("{}_{}", &input, i))?;
        let header = Header {
            length: input_length,
            num_shares,
            min_shares,
            evaluation_point: i as u64,
        };
        let ser_header = bincode::serialize(&header)?;
        file.write_all(&ser_header)?;
        output_files.push(file);
    }

    let ibytes = U64Iterator::new(input_file.bytes());
    for bres in ibytes {
        let b = bres?;
        let polynomial = Polynomial::random(min_shares, b);
        let points: Vec<u64> = (1..=num_shares as u64).collect();
        let evaluations = polynomial.evaluate_at(&points);
        for i in 0..num_shares {
            output_files[i].write_all(&u64::to_le_bytes(evaluations[i]))?;
        }
    }

    for i in 0..num_shares {
        output_files[i].flush()?;
        output_files[i].sync_all()?;
    }
    Ok(())
}

pub fn decode(paths: &[String], output_path: String) -> Result<(), Box<dyn Error>> {
    let mut first_file = File::open(&paths[0])?;
    let stub_header = Header {
        length: 0,
        num_shares: 0,
        min_shares: 0,
        evaluation_point: 0,
    };
    let header_size = bincode::serialized_size(&stub_header)?;
    let mut header_bytes = vec![0u8; header_size as usize];
    first_file.read_exact(&mut header_bytes)?;

    let header: Header = bincode::deserialize(&header_bytes)?;

    if paths.len() < header.min_shares {
        panic!("Too few shares avaialble for reconstruction");
    }

    let mut evaluation_points = vec![header.evaluation_point];
    let mut iterators = vec![U64Iterator::new(first_file.bytes())];

    for i in 1..header.min_shares {
        let mut file = File::open(&paths[i])?;
        file.read_exact(&mut header_bytes)?;
        let header: Header = bincode::deserialize(&header_bytes)?;

        evaluation_points.push(header.evaluation_point);
        iterators.push(U64Iterator::new(file.bytes()));
    }

    let lagrange_basis = Polynomial::compute_lagrange_basis_for_constant_term(&evaluation_points);
    let number_of_iterations = header.length / 8;
    let spare_bytes = (header.length % 8) as usize;

    let mut evaluations = vec![0u64; header.min_shares];
    let mut output_file = File::create(output_path)?;
    for _ in 0..number_of_iterations {
        for i in 0..header.min_shares {
            evaluations[i] = iterators[i].next().ok_or(0).unwrap()?;
        }
        let data = Polynomial::interpolate_from_langrange_basis(&evaluations, &lagrange_basis);
        let output_data = data.to_le_bytes();
        output_file.write_all(&output_data)?;
    }

    if spare_bytes != 0 {
        for i in 0..header.min_shares {
            evaluations[i] = iterators[i].next().ok_or(0).unwrap()?;
        }
        let data = Polynomial::interpolate_from_langrange_basis(&evaluations, &lagrange_basis);

        let output_data = data.to_le_bytes();
        output_file.write_all(&output_data[0..spare_bytes])?;
    }

    Ok(())
}
