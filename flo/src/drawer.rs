use flo_draw::canvas::{
    curves::{bezier::path::BezierPath, BezierCurve},
    *,
};
use smallvec::*;
use std::io;
use std::iter;
use std::sync::*;

use image;
use image::io::Reader as ImageReader;

pub struct Drawer<'a> {
    pub gc: &'a mut Vec<Draw>,
    pub height: f32,
}

impl<'a> GraphicsContext for Drawer<'a> {
    /// Suspends rendering to the display until the next 'ShowFrame'
    ///
    /// The renderer may perform tessellation or rendering in the background after 'StartFrame' but won't
    /// commit anything to the visible frame buffer until 'ShowFrame' is hit. If 'StartFrame' is nested,
    /// then the frame won't be displayed until 'ShowFrame' has been requested at least that many times.
    ///
    /// The frame state persists across a 'ClearCanvas'
    fn start_frame(&mut self) {
        self.draw(Draw::StartFrame);
    }

    /// Displays any requested queued after 'StartFrame'
    fn show_frame(&mut self) {
        self.draw(Draw::ShowFrame);
    }

    /// Resets the frame count back to 0 (for when regenerating the state of a canvas)
    fn reset_frame(&mut self) {
        self.draw(Draw::ResetFrame);
    }

    /// Begins a new path
    fn new_path(&mut self) {
        self.draw(Draw::Path(PathOp::NewPath));
    }

    /// Move to a new point in the current path (paths should always begin with a move instruction, and moves can define subpaths)
    fn move_to(&mut self, x: f32, y: f32) {
        self.draw(Draw::Path(PathOp::Move(x, self.height - y)));
    }

    /// Adds a line to the current path
    fn line_to(&mut self, x: f32, y: f32) {
        self.draw(Draw::Path(PathOp::Line(x, self.height - y)));
    }

    /// Adds a bezier curve to the current path
    fn bezier_curve_to(&mut self, x: f32, y: f32, cp1_x: f32, cp1_y: f32, cp2_x: f32, cp2_y: f32) {
        self.draw(Draw::Path(PathOp::BezierCurve(
            ((cp1_x, self.height - cp1_y), (cp2_x, self.height - cp2_y)),
            (x, self.height - y),
        )));
    }

    /// Closes the current path (adds a line to the last move point)
    fn close_path(&mut self) {
        self.draw(Draw::Path(PathOp::ClosePath));
    }

    /// Fills the currently defined path
    fn fill(&mut self) {
        self.draw(Draw::Fill);
    }

    /// Draws a line around the currently defined path
    fn stroke(&mut self) {
        self.draw(Draw::Stroke);
    }

    /// Sets the line width for the next stroke() operation
    fn line_width(&mut self, width: f32) {
        self.draw(Draw::LineWidth(width));
    }

    /// Sets the line width for the next stroke() operation in device pixels
    fn line_width_pixels(&mut self, width: f32) {
        self.draw(Draw::LineWidthPixels(width));
    }

    /// Sets the line join style for the next stroke() operation
    fn line_join(&mut self, join: LineJoin) {
        self.draw(Draw::LineJoin(join));
    }

    /// Sets the style of the start and end cap of the next line drawn by the stroke() operation
    fn line_cap(&mut self, cap: LineCap) {
        self.draw(Draw::LineCap(cap));
    }

    /// Sets the winding rule used to determine if an internal subpath should be filled or empty
    fn winding_rule(&mut self, winding_rule: WindingRule) {
        self.draw(Draw::WindingRule(winding_rule));
    }

    /// Starts defining a new dash pattern
    fn new_dash_pattern(&mut self) {
        self.draw(Draw::NewDashPattern);
    }

    /// Adds a dash of the specified length to the dash pattern (alternating between drawing and gap lengths)
    fn dash_length(&mut self, length: f32) {
        self.draw(Draw::DashLength(length));
    }

    /// Sets the offset for where the dash pattern starts at the next stroke
    fn dash_offset(&mut self, offset: f32) {
        self.draw(Draw::DashOffset(offset));
    }

    /// Sets the colour of the next fill() operation
    fn fill_color(&mut self, col: Color) {
        self.draw(Draw::FillColor(col));
    }

    /// Sets the texture to use for the next fill() operation
    ///
    /// The coordinates here specify the lower-left and upper-left position on the canvas where the texture will appear.
    /// Note that `fill_transform()` can be used to further rotate or scale the texture.
    fn fill_texture(&mut self, texture_id: TextureId, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.draw(Draw::FillTexture(
            texture_id,
            (x1, self.height - y1),
            (x2, self.height - y2),
        ));
    }

    /// Sets the gradient to use for the next fill() operation
    fn fill_gradient(&mut self, gradient_id: GradientId, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.draw(Draw::FillGradient(
            gradient_id,
            (x1, self.height - y1),
            (x2, self.height - y2),
        ));
    }

    /// Applies a transformation to the fill texture or gradient
    fn fill_transform(&mut self, transform: Transform2D) {
        self.draw(Draw::FillTransform(transform));
    }

    /// Sets the colour to use for the next stroke() operation
    fn stroke_color(&mut self, col: Color) {
        self.draw(Draw::StrokeColor(col));
    }

    /// Sets the blend mode of the next fill or stroke operation
    fn blend_mode(&mut self, mode: BlendMode) {
        self.draw(Draw::BlendMode(mode));
    }

    /// Reset the canvas transformation to the identity transformation (so that the y axis goes from -1 to 1)
    fn identity_transform(&mut self) {
        self.draw(Draw::IdentityTransform);
    }

    /// Sets a transformation such that:
    /// (0,0) is the center point of the canvas
    /// (0,height/2) is the top of the canvas
    /// Pixels are square
    fn canvas_height(&mut self, height: f32) {
        self.draw(Draw::CanvasHeight(height));
    }

    /// Moves a particular region to the center of the canvas (coordinates are minx, miny, maxx, maxy)
    fn center_region(&mut self, minx: f32, miny: f32, maxx: f32, maxy: f32) {
        self.draw(Draw::CenterRegion((minx, miny), (maxx, maxy)));
    }

    /// Multiply a 2D transform by the current transformation
    fn transform(&mut self, transform: Transform2D) {
        self.draw(Draw::MultiplyTransform(transform));
    }

    /// Removes the current clipping path
    fn unclip(&mut self) {
        self.draw(Draw::Unclip);
    }

    /// Sets the current path as the clipping path
    fn clip(&mut self) {
        self.draw(Draw::Clip);
    }

    /// Stores the current contents of the canvas in a background buffer
    fn store(&mut self) {
        self.draw(Draw::Store);
    }

    /// Restores the contents of the canvas from the background buffer
    fn restore(&mut self) {
        self.draw(Draw::Restore);
    }

    /// Releases the memory allocated by the last store() operation
    fn free_stored_buffer(&mut self) {
        self.draw(Draw::FreeStoredBuffer);
    }

    /// Stores the current state of the canvas (line width, fill colour, etc)
    fn push_state(&mut self) {
        self.draw(Draw::PushState);
    }

    /// Restore a state previously pushed
    ///
    /// This will restore the line width (and the other stroke settings), stroke colour, current path, fill colour,
    /// winding rule, sprite settings and blend settings.
    ///
    /// The currently selected layer is not affected by this operation.
    fn pop_state(&mut self) {
        self.draw(Draw::PopState);
    }

    /// Clears the canvas entirely to a background colour, and removes any stored resources (layers, sprites, fonts, textures)
    fn clear_canvas(&mut self, color: Color) {
        self.draw(Draw::ClearCanvas(color));
    }

    /// Selects a particular layer for drawing
    /// Layer 0 is selected initially. Layers are drawn in order starting from 0.
    /// Layer IDs don't have to be sequential.
    fn layer(&mut self, layer_id: LayerId) {
        self.draw(Draw::Layer(layer_id));
    }

    /// Sets how a particular layer is blended with the underlying layer
    fn layer_blend(&mut self, layer_id: LayerId, blend_mode: BlendMode) {
        self.draw(Draw::LayerBlend(layer_id, blend_mode));
    }

    /// Sets the alpha value for a particular layer
    fn layer_alpha(&mut self, layer_id: LayerId, alpha: f64) {
        self.draw(Draw::LayerAlpha(layer_id, alpha as _));
    }

    /// Clears the current layer
    fn clear_layer(&mut self) {
        self.draw(Draw::ClearLayer);
    }

    /// Clears all of the layers (without resetting any other resources, as clear_canvas does)
    fn clear_all_layers(&mut self) {
        self.draw(Draw::ClearAllLayers);
    }

    /// Exchanges the contents of two layers in the drawing
    fn swap_layers(&mut self, layer1: LayerId, layer2: LayerId) {
        self.draw(Draw::SwapLayers(layer1, layer2));
    }

    /// Selects a particular sprite for drawing
    ///
    /// Future drawing actions are sent to this sprite: use something like `Layer(0)` to start drawing
    /// to a layer again.
    ///
    /// Sprites can be repeatedly re-rendered with a single command and their appearance may be
    /// cached for efficiency. Actions that affect the whole canvas or layers are not permitted in
    /// sprites.
    fn sprite(&mut self, sprite_id: SpriteId) {
        self.draw(Draw::Sprite(sprite_id));
    }

    /// Releases the resources used by the current sprite
    fn clear_sprite(&mut self) {
        self.draw(Draw::ClearSprite);
    }

    /// Adds a sprite transform to the next sprite drawing operation
    fn sprite_transform(&mut self, transform: SpriteTransform) {
        self.draw(Draw::SpriteTransform(transform));
    }

    /// Renders a sprite with the transformations set by `sprite_transform()`
    fn draw_sprite(&mut self, sprite_id: SpriteId) {
        self.draw(Draw::DrawSprite(sprite_id));
    }

    /// Renders a sprite to a texture, then applies a set of filters before committing to the drawing
    ///
    /// (Unlike a dynamic texture, the texture isn't retained and the effect is reapplied every time the scene is rendered)
    fn draw_sprite_with_filters(&mut self, sprite_id: SpriteId, filters: Vec<TextureFilter>) {
        self.draw(Draw::DrawSpriteWithFilters(sprite_id, filters));
    }

    /// Moves the definition from the specified sprite to this one (faster than copying)
    fn move_sprite_from(&mut self, source_sprite_id: SpriteId) {
        self.draw(Draw::MoveSpriteFrom(source_sprite_id));
    }

    /// Loads font data into the canvas for a particular font ID
    fn define_font_data(&mut self, font_id: FontId, font_data: Arc<CanvasFontFace>) {
        self.draw(Draw::Font(font_id, FontOp::UseFontDefinition(font_data)));
    }

    /// Sets the size that text in the specified font will be rendered at
    fn set_font_size(&mut self, font_id: FontId, size: f32) {
        self.draw(Draw::Font(font_id, FontOp::FontSize(size)));
    }

    /// Draws a text string using a font
    /// The point (baseline_x, baseline_y) is the left __bottom__ corner of first glyph
    fn draw_text(&mut self, font_id: FontId, text: String, baseline_x: f32, baseline_y: f32) {
        self.draw(Draw::DrawText(
            font_id,
            text,
            baseline_x,
            self.height - baseline_y,
        ));
    }

    /// Draws specific glyphs from a font
    fn draw_glyphs(&mut self, font_id: FontId, glyphs: Vec<GlyphPosition>) {
        self.draw(Draw::Font(font_id, FontOp::DrawGlyphs(glyphs)));
    }

    /// Starts laying out a line of text
    fn begin_line_layout(&mut self, x: f32, y: f32, align: TextAlignment) {
        self.draw(Draw::BeginLineLayout(x, self.height - y, align));
    }

    /// Adds text to the current line layout
    fn layout_text(&mut self, font_id: FontId, text: String) {
        self.draw(Draw::Font(font_id, FontOp::LayoutText(text)));
    }

    /// Finishes laying out text and renders the result
    fn draw_text_layout(&mut self) {
        self.draw(Draw::DrawLaidOutText);
    }

    /// Creates a new texture that can be used with fill_texture of the specified width and height
    fn create_texture(
        &mut self,
        texture_id: TextureId,
        width: u32,
        height: u32,
        format: TextureFormat,
    ) {
        self.draw(Draw::Texture(
            texture_id,
            TextureOp::Create(TextureSize(width, height), format),
        ));
    }

    /// Releases the memory allocated to a particular texture
    fn free_texture(&mut self, texture_id: TextureId) {
        self.draw(Draw::Texture(texture_id, TextureOp::Free));
    }

    /// Sets the bitmap data for a texture, in the format specified by the call to create_texture()
    fn set_texture_bytes(
        &mut self,
        texture_id: TextureId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        bytes: Arc<Vec<u8>>,
    ) {
        self.draw(Draw::Texture(
            texture_id,
            TextureOp::SetBytes(TexturePosition(x, y), TextureSize(width, height), bytes),
        ));
    }

    /// Creates the texture bytes by drawing from a sprite
    fn set_texture_from_sprite(
        &mut self,
        texture_id: TextureId,
        sprite_id: SpriteId,
        sprite_x: f32,
        sprite_y: f32,
        sprite_width: f32,
        sprite_height: f32,
    ) {
        self.draw(Draw::Texture(
            texture_id,
            TextureOp::SetFromSprite(
                sprite_id,
                SpriteBounds(
                    SpritePosition(sprite_x, sprite_y),
                    SpriteSize(sprite_width, sprite_height),
                ),
            ),
        ));
    }

    /// Creates a dynamic texture that is rendered from a sprite and automatically chooses its resolution to cover
    /// a particular area of the canvas.
    ///
    /// This is useful for applying filter effects to rendering, or caching a complicated rendering for later. It can
    /// also be used to update parts of the rendering just by changing the source sprite, and can be combined with
    /// filters such as the gaussian blur filter for more complicated effects.
    fn create_dynamic_texture(
        &mut self,
        texture_id: TextureId,
        sprite_id: SpriteId,
        sprite_x: f32,
        sprite_y: f32,
        sprite_width: f32,
        sprite_height: f32,
        canvas_width: f32,
        canvas_height: f32,
    ) {
        self.draw(Draw::Texture(
            texture_id,
            TextureOp::CreateDynamicSprite(
                sprite_id,
                SpriteBounds(
                    SpritePosition(sprite_x, sprite_y),
                    SpriteSize(sprite_width, sprite_height),
                ),
                CanvasSize(canvas_width, canvas_height),
            ),
        ));
    }

    /// Applies an alpha value to a texture
    fn set_texture_fill_alpha(&mut self, texture_id: TextureId, alpha: f32) {
        self.draw(Draw::Texture(
            texture_id,
            TextureOp::FillTransparency(alpha),
        ));
    }

    /// Copies a texture from one ID to another
    fn copy_texture(&mut self, source_texture_id: TextureId, target_texture_id: TextureId) {
        self.draw(Draw::Texture(
            source_texture_id,
            TextureOp::Copy(target_texture_id),
        ));
    }

    ///
    /// Applies a filter to a texture (see `TextureFilter` for a list of choices)
    ///
    fn filter_texture(&mut self, texture_id: TextureId, filter: TextureFilter) {
        self.draw(Draw::Texture(texture_id, TextureOp::Filter(filter)));
    }

    /// Applies a gaussian blur to a texture
    ///
    /// The radius is measured in texture units: for a standard texture, this is just pixels but for a dynamic texture, this
    /// is in canvas coordinates (so the blur effect doesn't change if the canvas is resized)
    ///
    /// The standard deviation for a blur created using this filter is 0.25 * radius
    fn gaussian_blur_texture(&mut self, texture_id: TextureId, radius: f32) {
        self.draw(Draw::Texture(
            texture_id,
            TextureOp::Filter(TextureFilter::GaussianBlur(radius)),
        ));
    }

    /// Defines a new gradient with a colour at stop position 0.0. Gradients can be used via fill_gradient()
    fn create_gradient(&mut self, gradient_id: GradientId, initial_color: Color) {
        self.draw(Draw::Gradient(
            gradient_id,
            GradientOp::Create(initial_color),
        ));
    }

    /// Adds a new colour stop to a texture
    fn gradient_stop(&mut self, gradient_id: GradientId, pos: f32, color: Color) {
        self.draw(Draw::Gradient(gradient_id, GradientOp::AddStop(pos, color)));
    }

    /// Sends a single drawing instruction to this graphics context
    fn draw(&mut self, d: Draw) {
        self.gc.draw(d);
    }
}

impl<'a> GraphicsPrimitives for Drawer<'a> {
    ///
    /// Draws a rectangle between particular coordinates
    ///
    fn rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        for d in draw_rect(x1, self.height - y2, x2, self.height - y1) {
            self.draw(d);
        }
    }

    ///
    /// Draws a circle at a particular point
    ///
    fn circle(&mut self, center_x: f32, center_y: f32, radius: f32) {
        for d in draw_circle(center_x, self.height - center_y, radius) {
            self.draw(d);
        }
    }

    ///
    /// Draws a bezier path
    ///
    fn bezier_path<TPath>(&mut self, path: &TPath)
    where
        TPath: BezierPath,
        TPath::Point: Coordinate2D,
    {
        let start_point = path.start_point();

        self.move_to(start_point.x() as _, self.height - (start_point.y() as f32));
        for (cp1, cp2, end) in path.points() {
            self.bezier_curve_to(
                end.x() as _,
                end.y() as _,
                cp1.x() as _,
                cp1.y() as _,
                cp2.x() as _,
                cp2.y() as _,
            );
        }
    }

    ///
    /// Renders the current path using a set of path attributes
    ///
    fn render_with_attributes<'b>(
        &'b mut self,
        attributes: impl 'b + IntoIterator<Item = &'b PathAttribute>,
    ) {
        let height = self.height;
        self.draw_list(
            attributes
                .into_iter()
                .flat_map(|attr| -> SmallVec<[Draw; 4]> {
                    match attr {
                        PathAttribute::Fill(color) => {
                            smallvec![Draw::FillColor(*color), Draw::Fill,]
                        }

                        PathAttribute::Stroke(width, color) => smallvec![
                            Draw::LineWidth(*width),
                            Draw::StrokeColor(*color),
                            Draw::Stroke,
                        ],

                        PathAttribute::StrokePixels(width, color) => smallvec![
                            Draw::LineWidthPixels(*width),
                            Draw::StrokeColor(*color),
                            Draw::Stroke,
                        ],

                        PathAttribute::FillGradient(
                            gradient,
                            (x1, y1),
                            (x2, y2),
                            Some(transform),
                        ) => smallvec![
                            Draw::FillGradient(*gradient, (*x1, height - *y1), (*x2, height - *y2)),
                            Draw::FillTransform(*transform),
                            Draw::Fill,
                        ],

                        PathAttribute::FillGradient(gradient, (x1, y1), (x2, y2), None) => {
                            smallvec![
                                Draw::FillGradient(
                                    *gradient,
                                    (*x1, height - *y2),
                                    (*x2, height - *y1)
                                ),
                                Draw::FillTransform(Transform2D::identity()),
                                Draw::Fill,
                            ]
                        }

                        PathAttribute::FillTexture(
                            texture,
                            (x1, y1),
                            (x2, y2),
                            Some(transform),
                        ) => smallvec![
                            Draw::FillTexture(*texture, (*x1, height - *y2), (*x2, height - *y1)),
                            Draw::FillTransform(*transform),
                            Draw::Fill,
                        ],

                        PathAttribute::FillTexture(texture, (x1, y1), (x2, y2), None) => smallvec![
                            Draw::FillTexture(*texture, (*x1, height - *y2), (*x2, height - *y1)),
                            Draw::FillTransform(Transform2D::identity()),
                            Draw::Fill,
                        ],
                    }
                }),
        );
    }

    ///
    /// Renders a bezier shape described by one or more paths and a set of attributes
    ///
    /// This can be used to quickly render the output of `drawing_to_attributed_paths()`
    ///
    fn render_bezier_shape<'b, TPath>(
        &'b mut self,
        attributes: impl 'b + IntoIterator<Item = &'b PathAttribute>,
        path_set: impl 'b + IntoIterator<Item = &'b TPath>,
    ) where
        TPath: 'b + BezierPath,
        TPath::Point: Coordinate2D,
    {
        self.new_path();

        for path in path_set {
            self.bezier_path(path);
        }

        self.render_with_attributes(attributes);
    }

    ///
    /// Draws a bezier curve (defined by the BezierCurve trait)
    ///
    fn bezier_curve<TCurve: BezierCurve>(&mut self, curve: &TCurve)
    where
        TCurve::Point: Coordinate2D,
    {
        let (cp1, cp2) = curve.control_points();
        let end = curve.end_point();

        self.bezier_curve_to(
            end.x() as _,
            end.y() as _,
            cp1.x() as _,
            cp1.y() as _,
            cp2.x() as _,
            cp2.y() as _,
        );
    }

    ///
    /// Draws a series of instructions
    ///
    fn draw_list<'b>(&'b mut self, drawing: impl 'b + IntoIterator<Item = Draw>) {
        for d in drawing.into_iter() {
            self.draw(d);
        }
    }

    ///
    /// Loads an image from an IO stream into a texture, returning the size (or None if the image can't be read for any reason)
    ///
    fn load_texture<TSrc: io::BufRead + io::Read + io::Seek>(
        &mut self,
        texture_id: TextureId,
        data: TSrc,
    ) -> Option<(usize, usize)> {
        // Load the image
        let img = ImageReader::new(data).with_guessed_format().ok()?;
        let img = img.decode().ok()?;

        // Convert to 8-bit RGBA
        let img = img.into_rgba8();
        let width = img.width();
        let height = img.height();

        // Load the texture
        let raw_pixels = Arc::new(img.into_raw());
        self.create_texture(texture_id, width, height, TextureFormat::Rgba);
        self.set_texture_bytes(texture_id, 0, 0, width, height, raw_pixels);

        // Result is the image size
        Some((width as _, height as _))
    }
}
