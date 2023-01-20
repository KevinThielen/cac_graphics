use std::fmt::Display;

/// Range in which color channels are assumed to be equal, to avoid floating point accuracies.
/// For example, the difference between RGB(0.92, 0.32, 0.34) and RGB(0.91, 0.33, 0.33) is negligable

pub const EPSILON: f32 = f32::EPSILON;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    data: Data,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Data {
    U8(Vec<u8>),
    F32(Vec<f32>),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Format {
    RgbU8,
    RgbF32,
    RgbaU8,
    RgbaF32,
}

#[derive(Debug)]
pub enum Error {
    ConversionFailed(&'static str),
    FileNotFound(String),
    UpScalingNotSupported,
    DimensionMismatch,
    EncodingFailed(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConversionFailed(error) => write!(f, "conversion failed, caused by {error}"),
            Self::FileNotFound(p) => write!(f, "{p} not found"),
            Self::UpScalingNotSupported => write!(f, "upscaling is not supported yet"),
            Self::DimensionMismatch => write!(
                f,
                "not enough data provided for all channels with width/height"
            ),
            Self::EncodingFailed(e) => write!(f, "encoding failed, caused by {e}"),
        }
    }
}

impl Image {
    /// Constructor
    ///
    /// Creates a new Image from the paramters.
    ///
    /// # Errors
    /// `DimensionMismatch` is returned when the length of the data is less than width * height *
    /// channels of the format
    pub fn new(width: u32, height: u32, format: Format, data: Data) -> Result<Self, Error> {
        if data.len() < width as usize * height as usize * format.channels() as usize {
            Err(Error::DimensionMismatch)
        } else {
            Ok(Self {
                width,
                height,
                format,
                data,
            })
        }
    }

    /// Creates a new image with a specific Color32
    ///
    /// # Errors
    /// `ConversionFailed` when either the width or the height can't be converted into usize
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn with_color32(
        width: u32,
        height: u32,
        color: crate::color32::Color32,
        format: Format,
    ) -> Result<Self, Error> {
        let w: usize = width
            .try_into()
            .map_err(|_| Error::ConversionFailed("width to usize"))?;

        let h: usize = height
            .try_into()
            .map_err(|_| Error::ConversionFailed("height to usize"))?;

        let channels = format.channels().into();
        let len = w * h * channels;

        let data: Data = match format {
            Format::RgbU8 | Format::RgbaU8 => Data::U8(
                color.as_rgba()[0..channels]
                    .iter()
                    .map(|v| (v * 255.0).clamp(0.0, 255.0) as u8)
                    .cycle()
                    .take(len)
                    .collect(),
            ),
            Format::RgbF32 | Format::RgbaF32 => Data::F32(
                color.as_rgba()[0..channels]
                    .iter()
                    .copied()
                    .cycle()
                    .take(len)
                    .collect(),
            ),
        };

        Self::new(width, height, format, data)
    }

    /// Generate a 64 bit hash from the image, using a perceibed hash algorithm
    ///
    /// The images gets reduced to a 8x8 grayscale image and then calculates by comparing the
    /// pixel values. The Hamming distance of the hash should be pretty close, of not the same for
    /// two of the same images, even if their aspect ratio/scale is different.
    ///
    /// # Errors
    /// `UpScalingNotSupported` when the image is smaller than 8px wide or high
    ///
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn hash(&self) -> Result<u64, Error> {
        let img = if self.width != 8 || self.height != 8 {
            Some(self.resize(8, 8)?)
        } else {
            None
        };

        let data = img.as_ref().map_or(&self.data, |img| &img.data);

        let channels = self.format.channels().into();
        debug_assert!(data.len() == 8 * 8 * channels);

        let grey_scale: Vec<_> = match data {
            Data::U8(data) => data
                .chunks(channels)
                .map(|v| (v.iter().map(|v| f32::from(*v)).sum::<f32>() / v.len() as f32))
                .collect(),
            Data::F32(data) => data
                .chunks(channels)
                .map(|v| v.iter().sum::<f32>() / v.len() as f32)
                .collect(),
        };

        debug_assert!(grey_scale.len() == 64);

        let average = grey_scale.iter().sum::<f32>() / grey_scale.len() as f32;

        let mut hash = 0;
        for (i, value) in grey_scale.iter().enumerate() {
            hash |= u64::from((value - average) >= EPSILON) << i;
        }

        Ok(hash)
    }

    /// Scale the image using nearest neighbour, returning a new image
    ///
    ///
    /// # Errors
    /// `UpScalingNotSupported` when the new width/height are greater than the old one.
    /// `ConversionFailed` when either width or height can't be converted to usize
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn resize(&self, new_width: u32, new_height: u32) -> Result<Self, Error> {
        if self.width == new_width && self.height == new_height {
            return Ok(self.clone());
        } else if new_width > self.width || new_height > self.height {
            return Err(Error::UpScalingNotSupported);
        }

        let width: usize = new_width
            .try_into()
            .map_err(|_| Error::ConversionFailed("width to usize"))?;

        let height: usize = new_height
            .try_into()
            .map_err(|_| Error::ConversionFailed("height to usize"))?;

        let scale_x = new_width as f32 / self.width as f32;
        let scale_y = new_height as f32 / self.height as f32;

        let channels: usize = self.format.channels().into();

        let mut new_data = match &self.data {
            Data::F32(_) => Data::F32(vec![1.0; width * height * channels]),
            Data::U8(_) => Data::U8(vec![255; width * height * channels]),
        };

        for y in 0..height {
            for x in 0..width {
                let nearest_x: usize = (x as f32 / scale_x).round() as usize;
                let nearest_y = (y as f32 / scale_y).round() as usize;

                let old_index = nearest_x * channels + nearest_y * self.width as usize * channels;
                let new_index = x * channels + y * width * channels;

                match (&self.data, &mut new_data) {
                    (Data::U8(old_data), Data::U8(new_data)) => {
                        let pixel = &old_data[old_index..][..channels];
                        new_data.splice(new_index..new_index + channels, pixel.iter().copied());
                    }
                    (Data::F32(old_data), Data::F32(new_data)) => {
                        let pixel = &old_data[old_index..][..channels];
                        new_data.splice(new_index..new_index + channels, pixel.iter().copied());
                    }
                    //literally creating the same "new data" with the same type of the old data above
                    //this loop, so this should never be reached
                    _ => unreachable!(),
                }
            }
        }
        Ok(Self {
            width: new_width,
            height: new_height,
            format: self.format,
            data: new_data,
        })
    }

    #[must_use]
    pub fn sample(&self, pixel_x: u32, pixel_y: u32) -> Option<Pixel> {
        if pixel_x >= self.width || pixel_y >= self.height {
            None
        } else {
            let channels: u32 = self.format.channels().into();
            let index = pixel_y * self.width * channels + pixel_x * channels;
            let index: usize = match index.try_into() {
                Ok(v) => v,
                Err(_) => return None,
            };

            let channels: usize = match channels.try_into() {
                Ok(v) => v,
                Err(_) => return None,
            };

            match &self.data {
                Data::U8(data) => {
                    let pixel_color = &data[index..index + channels];
                    pixel_color.try_into().ok()
                }
                Data::F32(data) => {
                    let pixel_color = &data[index..index + channels];

                    pixel_color.try_into().ok()
                }
            }
        }
    }

    /// Constructor
    /// Creates a new image from the raw bytes of an image file.
    /// Currently, only the png format is supported.
    /// For raw pixel data, see `new` instead.
    ///
    /// # Errors
    /// `FileNotFound` when the bytes don't match a valid file
    pub fn load_from_memory(format: Format, data: &[u8]) -> Result<Self, Error> {
        let img = image::load_from_memory(data);

        match img {
            Ok(img) => {
                let data = match format {
                    Format::RgbU8 => Data::U8(img.to_rgb8().to_vec()),
                    Format::RgbaU8 => Data::U8(img.to_rgba8().to_vec()),
                    Format::RgbF32 => Data::F32(img.to_rgb32f().to_vec()),
                    Format::RgbaF32 => Data::F32(img.to_rgba32f().to_vec()),
                };

                Self::new(img.width(), img.height(), format, data)
            }
            Err(e) => Err(Error::FileNotFound(e.to_string())),
        }
    }

    /// Constructor
    /// Creates a new image from an image file.
    /// Currently, only the png format is supported.
    ///
    /// # Errors
    /// `FileNotFound` when the file can't be found/loaded.
    pub fn load_from_file(
        format: Format,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Error> {
        let img = image::open(path);

        match img {
            Ok(img) => {
                let data = match format {
                    Format::RgbU8 => Data::U8(img.to_rgb8().to_vec()),
                    Format::RgbaU8 => Data::U8(img.to_rgba8().to_vec()),
                    Format::RgbF32 => Data::F32(img.to_rgb32f().to_vec()),
                    Format::RgbaF32 => Data::F32(img.to_rgba32f().to_vec()),
                };

                Self::new(img.width(), img.height(), format, data)
            }
            Err(e) => Err(Error::FileNotFound(e.to_string())),
        }
    }

    /// Saves the image to disk in the png format.
    ///
    /// # Errors
    /// `EncodingFailed` when the image couldn't be encoded into one of the supported file
    /// formats(right now it's only png)
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let img_data = match &self.data {
            Data::U8(data) => data.clone(),
            Data::F32(data) => data
                .iter()
                .map(|v| ((255.0 * v.clamp(0.0, 255.0)) as u8))
                .collect(),
        };

        match self.format.channels() {
            4 => image::RgbaImage::from_vec(self.width, self.height, img_data)
                .ok_or(Error::ConversionFailed("rgba image from f32 source data"))?
                .save_with_format(path, image::ImageFormat::Png)
                .map_err(|e| Error::EncodingFailed(e.to_string())),
            3 => image::RgbImage::from_vec(self.width, self.height, img_data)
                .ok_or(Error::ConversionFailed("rgb image from f32 source data"))?
                .save_with_format(path, image::ImageFormat::Png)
                .map_err(|e| Error::EncodingFailed(e.to_string())),
            n => Err(Error::EncodingFailed(format!(
                "channel count not supported ({n})",
            ))),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Pixel {
    RgbaF32([f32; 4]),
    RgbF32([f32; 3]),
    RgbU8([u8; 3]),
    RgbaU8([u8; 4]),
}

impl PartialEq for Pixel {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::RgbF32(s), Self::RgbF32(o)) => {
                !s.iter().zip(o.iter()).any(|(s, o)| (s - o).abs() > EPSILON)
            }
            (Self::RgbaF32(s), Self::RgbaF32(o)) => {
                !s.iter().zip(o.iter()).any(|(s, o)| (s - o).abs() > EPSILON)
            }
            (Self::RgbU8(s), Self::RgbU8(o)) => !s.iter().zip(o.iter()).any(|(s, o)| s != o),
            (Self::RgbaU8(s), Self::RgbaU8(o)) => !s.iter().zip(o.iter()).any(|(s, o)| s != o),
            _ => false,
        }
    }
}

impl Data {
    pub fn as_mut_ptr<T>(&mut self) -> *mut T {
        match self {
            Self::U8(data) => data.as_mut_ptr().cast(),
            Self::F32(data) => data.as_mut_ptr().cast(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::U8(data) => data.len(),
            Self::F32(data) => data.len(),
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::U8(data) => data.is_empty(),
            Self::F32(data) => data.is_empty(),
        }
    }
}
impl Format {
    #[must_use]
    pub const fn channels(&self) -> u8 {
        match self {
            Self::RgbU8 | Self::RgbF32 => 3,
            Self::RgbaU8 | Self::RgbaF32 => 4,
        }
    }

    #[must_use]
    pub fn create_storage(&self, pixel_count: usize) -> Data {
        let c = self.channels() as usize;
        match self {
            Self::RgbaU8 | Self::RgbU8 => Data::U8(vec![0; pixel_count * c]),
            Self::RgbF32 | Self::RgbaF32 => Data::F32(vec![0.0; pixel_count * c]),
        }
    }
}

impl TryFrom<&[f32]> for Pixel {
    type Error = Error;

    fn try_from(value: &[f32]) -> Result<Self, Self::Error> {
        match value.len() {
            3 => Ok(Self::RgbF32([value[0], value[1], value[2]])),
            4 => Ok(Self::RgbaF32([value[0], value[1], value[2], value[3]])),
            _ => Err(Error::ConversionFailed(
                "[f32] to Pixel have different channels",
            )),
        }
    }
}

impl TryFrom<&[u8]> for Pixel {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.len() {
            3 => Ok(Self::RgbU8([value[0], value[1], value[2]])),
            4 => Ok(Self::RgbaU8([value[0], value[1], value[2], value[3]])),
            _ => Err(Error::ConversionFailed(
                "[u8] to Pixel have different channels",
            )),
        }
    }
}
impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.format == other.format
            && self.data == other.data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn load_image_hash(format: Format, data: &[u8]) -> u64 {
        let img = image::load_from_memory(data).unwrap();

        let data = match format {
            Format::RgbU8 => Data::U8(img.to_rgb8().to_vec()),
            Format::RgbF32 => Data::F32(img.to_rgb32f().to_vec()),
            Format::RgbaU8 => Data::U8(img.to_rgba8().to_vec()),
            Format::RgbaF32 => Data::F32(img.to_rgba32f().to_vec()),
        };
        let image = Image::new(img.width(), img.height(), format, data).unwrap();

        let image = image.resize(8, 8).unwrap();
        image.hash().unwrap()
    }

    fn hash_test_format(format: Format) {
        let original_image = include_bytes!("../res/ferris.png");
        let small_image = include_bytes!("../res/ferris_small.png");
        let medium_image = include_bytes!("../res/ferris_medium.png");
        let not_ferris = include_bytes!("../res/not_ferris.png");
        let random_image = include_bytes!("../res/cute_anime_girl.png");

        let original_hash = load_image_hash(format, original_image);
        let small_hash = load_image_hash(format, small_image);
        let medium_hash = load_image_hash(format, medium_image);
        let not_ferris_hash = load_image_hash(format, not_ferris);
        let random_hash = load_image_hash(format, random_image);

        //hamming distance to check how different the structure of the images are.
        //only a tiny difference, despite the different aspect ratios and sizes
        let d = (original_hash ^ small_hash).count_ones();
        assert!(d <= 1);

        let d = (medium_hash ^ small_hash).count_ones();
        assert!(d <= 1);

        let d = (medium_hash ^ original_hash).count_ones();
        assert!(d <= 1);

        //slightly different image
        let d = (original_hash ^ not_ferris_hash).count_ones();
        assert!(
            d >= 2,
            "d is {d} \n{original_hash:b}^ \n{not_ferris_hash:b}"
        );

        //vastly different image
        let d = (original_hash ^ random_hash).count_ones();
        assert!(d > 20, "d is {d} {random_hash} ^ {original_hash}");
    }

    #[test]
    fn hash_test_f32() {
        hash_test_format(Format::RgbF32);
        hash_test_format(Format::RgbaF32);
    }

    #[test]
    fn hash_test_u8() {
        hash_test_format(Format::RgbU8);
        hash_test_format(Format::RgbaU8);
    }
}
