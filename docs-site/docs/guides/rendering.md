# Rendering Pages to Images

paperjam can render PDF pages to PNG, JPEG, or BMP images using the pdfium engine. This requires the `render` feature, which is included in all pre-built PyPI wheels.

## Rendering a single page

```python
import paperjam

doc = paperjam.open("slides.pdf")

# Render page 1 at 150 DPI (default)
img = doc.render_page(1)
img.save("page1.png")

# Higher resolution
img = doc.render_page(1, dpi=300, format="png")

# JPEG with quality setting
img = doc.render_page(1, dpi=150, format="jpeg", quality=90)
img.save("page1.jpg")

# White background (useful for pages with transparent areas)
img = doc.render_page(1, background_color=(255, 255, 255))
```

### Using the Page object

If you already have a `Page` object, call `render()` directly:

```python
page = doc.pages[0]
img = page.render(dpi=200, format="png")
img.save("cover.png")
```

## Rendering multiple pages

```python
# Render specific pages
images = doc.render_pages(pages=[1, 3, 5], dpi=150)

# Render all pages
images = doc.render_pages(dpi=96)

for img in images:
    img.save(f"page_{img.page:03d}.png")
    print(f"Page {img.page}: {img.width}×{img.height} px")
```

## Scaling by target dimensions

Instead of specifying DPI, you can target a specific pixel width or height. The other dimension is computed to preserve the aspect ratio:

```python
# Scale to 800px wide (height computed automatically)
img = doc.render_page(1, scale_to_width=800)

# Scale to 600px tall
img = doc.render_page(1, scale_to_height=600)
```

## Working with RenderedImage

`RenderedImage` is a frozen dataclass:

```python
img = doc.render_page(1)
print(img.width, img.height)   # pixel dimensions
print(img.format)              # "png", "jpeg", or "bmp"
print(img.page)                # 1-based page number
print(len(img.data))           # raw bytes size

# Save to disk
img.save("output.png")

# Use in-memory (e.g. pass to Pillow)
from PIL import Image as PILImage
import io
pil_img = PILImage.open(io.BytesIO(img.data))
```

`RenderedImage` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `data` | `bytes` | Raw image bytes in the specified format |
| `width` | `int` | Image width in pixels |
| `height` | `int` | Image height in pixels |
| `format` | `str` | `"png"`, `"jpeg"`, or `"bmp"` |
| `page` | `int` | 1-based source page number |

## Convenience function

The top-level `render()` function opens a file and renders a single page in one call:

```python
img = paperjam.render("slides.pdf", page=1, dpi=150)
img.save("slide1.png")
```

## Parameter reference

All render parameters are keyword-only (except `page_number`):

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `dpi` | `float` | `150` | Resolution in dots per inch |
| `format` | `str` | `"png"` | Output format: `"png"`, `"jpeg"`, `"bmp"` |
| `quality` | `int` | `85` | JPEG quality (1–100); ignored for PNG/BMP |
| `background_color` | `tuple \| None` | `None` | RGB background `(r, g, b)` each 0–255 |
| `scale_to_width` | `int \| None` | `None` | Target width in pixels (overrides `dpi`) |
| `scale_to_height` | `int \| None` | `None` | Target height in pixels (overrides `dpi`) |

## Async rendering

```python
# Render without blocking the event loop
img = await doc.arender_page(1, dpi=150)
images = await doc.arender_pages(pages=[1, 2, 3])
```
