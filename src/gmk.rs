use flate2::write::ZlibEncoder;
use gm8x::reader::Settings;
use gm8x::GameVersion;
use minio::WritePrimitives;
use std::io::{self, Write};
use std::u32;

pub trait WritePascalString: io::Write + minio::WritePrimitives {
    fn write_pas_string(&mut self, s: &str) -> io::Result<usize> {
        self.write_u32_le(s.len() as u32)
            .and_then(|x| self.write(s.as_bytes()).map(|y| y + x))
    }
}
impl<W> WritePascalString for W where W: io::Write {}

pub fn write_header<W>(
    writer: &mut W,
    version: GameVersion,
    game_id: u32,
    guid: [u32; 4],
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(1234321)?;
    result += writer.write_u32_le(match version {
        GameVersion::GameMaker8_0 => 800,
        GameVersion::GameMaker8_1 => 810,
    })?;
    result += writer.write_u32_le(game_id)?;
    for n in &guid {
        result += writer.write_u32_le(*n)?;
    }
    Ok(result)
}

pub fn write_settings<W>(
    writer: &mut W,
    settings: Settings,
    ico_file: Vec<u8>,
    version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    let mut enc = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_u32_le(settings.fullscreen as u32)?;
    enc.write_u32_le(settings.dont_draw_border as u32)?;
    enc.write_u32_le(settings.display_cursor as u32)?;
    enc.write_u32_le(settings.interpolate_pixels as u32)?;
    enc.write_i32_le(settings.scaling)?;
    enc.write_u32_le(settings.allow_resize as u32)?;
    enc.write_u32_le(settings.window_on_top as u32)?;
    enc.write_u32_le(settings.clear_colour)?;
    enc.write_u32_le(settings.set_resolution as u32)?;
    enc.write_u32_le(settings.colour_depth)?;
    enc.write_u32_le(settings.resolution)?;
    enc.write_u32_le(settings.frequency)?;
    enc.write_u32_le(settings.dont_show_buttons as u32)?;
    enc.write_u32_le(settings.vsync as u32)?;
    enc.write_u32_le(settings.disable_screensaver as u32)?;
    enc.write_u32_le(settings.f4_fullscreen_toggle as u32)?;
    enc.write_u32_le(settings.f1_help_menu as u32)?;
    enc.write_u32_le(settings.esc_close_game as u32)?;
    enc.write_u32_le(settings.f5_save_f6_load as u32)?;
    enc.write_u32_le(settings.f9_screenshot as u32)?;
    enc.write_u32_le(settings.treat_close_as_esc as u32)?;
    enc.write_u32_le(settings.priority)?;
    enc.write_u32_le(settings.freeze_on_lose_focus as u32)?;

    enc.write_u32_le(settings.loading_bar)?;
    if settings.loading_bar == 2 {
        // 2 = custom loading bar - otherwise don't write anything here

        match settings.backdata {
            Some(data) => {
                enc.write_u32_le(1)?;
                let mut backdata_enc = ZlibEncoder::new(
                    Vec::with_capacity(data.len()),
                    flate2::Compression::default(),
                );
                backdata_enc.write_all(&data)?;
                let compressed = backdata_enc.finish()?;
                enc.write_u32_le(compressed.len() as u32)?;
                enc.write_all(&compressed)?;
            }
            None => {
                enc.write_u32_le(0)?;
            }
        }

        match settings.frontdata {
            Some(data) => {
                enc.write_u32_le(1)?;
                let mut frontdata_enc = ZlibEncoder::new(
                    Vec::with_capacity(data.len()),
                    flate2::Compression::default(),
                );
                frontdata_enc.write_all(&data)?;
                let compressed = frontdata_enc.finish()?;
                enc.write_u32_le(compressed.len() as u32)?;
                enc.write_all(&compressed)?;
            }
            None => {
                enc.write_u32_le(0)?;
            }
        }
    }

    match settings.custom_load_image {
        Some(data) => {
            // In GMK format, the first bool is for whether there's a custom load image and the second is for
            // whether there's actually any data following it. There is only one bool in exe format, thus
            // we need to write two redundant "true"s here.
            enc.write_u32_le(1)?;
            enc.write_u32_le(1)?;
            let mut ci_enc = ZlibEncoder::new(
                Vec::with_capacity(data.len()),
                flate2::Compression::default(),
            );
            ci_enc.write_all(&data)?;
            let compressed = ci_enc.finish()?;
            enc.write_u32_le(compressed.len() as u32)?;
            enc.write_all(&compressed)?;
        }
        None => {
            enc.write_u32_le(0)?;
        }
    }

    enc.write_u32_le(settings.transparent as u32)?;
    enc.write_u32_le(settings.translucency)?;
    enc.write_u32_le(settings.scale_progress_bar as u32)?;

    enc.write_u32_le(ico_file.len() as u32)?;
    enc.write_all(&ico_file)?;

    enc.write_u32_le(settings.show_error_messages as u32)?;
    enc.write_u32_le(settings.log_errors as u32)?;
    enc.write_u32_le(settings.always_abort as u32)?;
    match version {
        GameVersion::GameMaker8_0 => enc.write_u32_le(settings.zero_uninitalized_vars as u32)?,
        GameVersion::GameMaker8_1 => enc.write_u32_le(
            ((settings.error_on_uninitalized_args as u32) << 1)
                | (settings.zero_uninitalized_vars as u32),
        )?,
    };

    enc.write_pas_string("decompiler clan :police_car: :police_car: :police_car:")?; // author
    enc.write_pas_string("")?; // version string
    enc.write_u64_le(0)?; // timestamp (actually an f64 but it doesn't matter what we write)
    enc.write_pas_string("")?; // information

    // TODO: extract all this stuff from .rsrc in gm8x
    enc.write_u32_le(1)?; // major version
    enc.write_u32_le(0)?; // minor version
    enc.write_u32_le(0)?; // release version
    enc.write_u32_le(0)?; // build version
    enc.write_pas_string("")?; // company
    enc.write_pas_string("")?; // product
    enc.write_pas_string("")?; // copyright info
    enc.write_pas_string("")?; // description
    enc.write_u64_le(0)?; // timestamp

    let encoded = enc.finish()?;
    result += writer.write_u32_le(encoded.len() as u32)?;
    result += writer.write(&encoded)?;

    Ok(result)
}