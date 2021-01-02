use std::io; //error::Error;

pub struct TransformPayload {
    ///* TODO DO we want support for multiple types at once? e.g. vector of dynamic trait objects?
    pub to_method_e: Option<Box<dyn _Encode>>,
    pub from_method_ed: Option<Box<dyn _Decode>>,
    pub to_method_c: Option<Box<dyn _Compress>>,
    pub from_method_cd: Option<Box<dyn _Decompress>>,
}

pub trait Transform {
    fn transform(&self, direction: bool, payload: &mut Vec<u8>) -> Result<(), io::Error>;
}

impl Transform for TransformPayload {
    fn transform(&self, direction: bool, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        match direction {
            true => match self.to_method_e.as_ref() {
                Some(e) => e._encode(payload),
                None => match self.to_method_c.as_ref() {
                    Some(e) => e._compress(payload),
                    None => Err(io::Error::new(io::ErrorKind::Other, "Not implemented")),
                },
            },
            false => match self.from_method_ed.as_ref() {
                Some(d) => d._decode(payload),
                None => match self.from_method_cd.as_ref() {
                    Some(d) => d._decompress(payload),
                    None => Err(io::Error::new(io::ErrorKind::Other, "Not implemented")),
                },
            },
        }
    }
}

pub trait _Encode {
    fn _encode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error>;
}

pub trait _Decode {
    fn _decode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error>;
}

pub trait _Compress {
    fn _compress(&self, payload: &mut Vec<u8>) -> Result<(), io::Error>;
}

pub trait _Decompress {
    fn _decompress(&self, payload: &mut Vec<u8>) -> Result<(), io::Error>;
}

/*
* The following are generic traits that
* users can implement dynamic trait objects on
* e.g. users can implement Encode for some algorithm like
* base 64. Then users can implement Encode<B: b64> for Vec<u8>
*


pub trait Compress<T> {
   // fn compress<T>(payload: Payload)
    fn _compress<T>(payload: Vec<u8>) -> Result<Vec<u8>, dyn Error>;
    fn __compress<T>(self) -> Result<Vec<u8>, dyn Error>;
    fn buf_compress<T>(self)-> Result<&[u8], dyn Error>;
}

pub trait Decompress<T> {
    fn decompress<T>(payload: Vec<u8>) -> Result<Vec<u8>, dyn Error>;
}

//Default associated type is self e.g. no encode
pub trait Encode<T=Self> {
    fn encode<T>(payload: Vec<u8>) -> Result<Vec<u8>, dyn Error>;
}
*/

pub trait ___Payload<E: _Encode> {
    type Input;
    fn encode<T>(&self, payload: &mut T) -> Result<(), io::Error>;
}

pub struct Payload {
    //<E: _Encode> {
    pub method: Box<dyn _Encode>,
}

impl Payload
//  where
//      E: _Encode,
{
    pub fn run(&self, bytes: &mut Vec<u8>) {
        println!("Running the outer method!\n");
        self.method._encode(bytes);
    }
}

pub struct EncodePayload {
    //<E: _Encode> {
    pub to_method: Box<dyn _Encode>,
    pub from_method: Box<dyn _Decode>,
}

impl EncodePayload
//  where
//      E: _Encode,
{
    pub fn run_to(&self, bytes: &mut Vec<u8>) {
        println!("Running the outer encode method!\n");
        self.to_method._encode(bytes);
    }
    pub fn run_from(&self, bytes: &mut Vec<u8>) {
        println!("Running the outer decode method!\n");
        self.from_method._decode(bytes);
    }
}

pub struct _Payload<T, F> {
    pub bytes: Vec<u8>,
    to_method: T,   //Box<dyn T>,
    from_method: F, //Box<dyn F>
}

impl<T, F> _Payload<T, F>
where
    T: _Encode,
    F: _Decode,
{
    pub fn run_to_encode(&self, bytes: &mut Vec<u8>) {
        self.to_method._encode(bytes);
    }
    pub fn run_from_encode(&self, bytes: &mut Vec<u8>) {
        self.from_method._decode(bytes);
    }
}

impl<T, F> _Payload<T, F>
where
    T: _Compress,
    F: _Decompress,
{
    pub fn run_to_compress(&self, bytes: &mut Vec<u8>) {
        self.to_method._compress(bytes);
    }
    pub fn run_from_compress(&self, bytes: &mut Vec<u8>) {
        self.from_method._decompress(bytes);
    }
}

/*
pub struct _Payload<'a, T> {
    pub bytes: Vec<u8>,
    pub bytes_unwrapped: &'a[u8],
    pub phantom: T
}
*/
/*
pub struct __Payload<E: Encode, D: Decode> {
    pub bytes: Vec<u8>,
    //pub bytes_unwrapped: &'a[u8],
}
*/

pub struct __Payload<T> {
    pub bytes: T, //Vec<u8>,
                  //pub bytes_unwrapped: &'a[u8],
}

#[derive(Copy, Clone)]
pub struct Default {}

impl _Encode for Default {
    fn _encode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner generic Encode method!!!\n");
        Ok(())
    }
}

impl _Decode for Default {
    fn _decode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner generic Decode method!!!\n");
        Ok(())
    }
}

impl _Compress for Default {
    fn _compress(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner generic Encode method!!!\n");
        Ok(())
    }
}

impl _Decompress for Default {
    fn _decompress(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner generic Decode method!!!\n");
        Ok(())
    }
}
