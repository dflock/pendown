# pendown::colours: the arithmetic

Status: done
Type: task

## What

A module without a window, `pendown::colours`:

- HSV (hue 0–360, saturation and brightness 0–1) to RGB 0–255 and back, with opacity alongside;
- the wheel: the hue and saturation at a point in a disc, and the point for a hue and saturation;
- the forms as Logo text: palette `4`, name `"tomato`, RGB `[255 99 71]`, RGBA `[255 99 71 128]`, hex `"#ff6347` (`"#ff634780` when not solid), lower case;
- parsing what's typed in the R, G, B, A and hex fields, with the reason when it doesn't make sense;
- the name or default palette number a colour is exactly, if any;
- the names in colour order: greys by lightness, then by hue.

## Done when

Unit tests cover each of these, including round trips and the wheel's edge and centre.

## Comments

### 2026-10-29: done

`crates/pendown/src/colours.rs`: `Hsva` and its round trip with RGBA, `wheel_point` and `wheel_colour` (red at the top, clockwise), `Form` and `text`, `name_of`, `palette_of` and `forms`, `parse_channel` and `parse_hex` with their reasons, `insertion` (the space before), and `names_by_colour` and `names_matching`. Seven unit tests.
