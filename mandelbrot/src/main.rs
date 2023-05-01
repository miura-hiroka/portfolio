use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::{WindowResized, WindowResolution},
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}
};

#[derive(Component)]
struct ImageFractal;

#[derive(Component)]
struct FPSText;

#[derive(Component)]
struct WorldFunc {
    f: fn(f64, f64) -> RGBA,
}

#[derive(Component)]
struct View {
    px_size: f64,
    nw: u32,
    nh: u32,
    x: f64,
    y: f64,
    // cache
    w: f64,
    h: f64,
    x0: f64,
    y0: f64,
}

impl View {
    fn new(px_size: f64, nw: u32, nh: u32, x: f64, y: f64) -> Self {
        let w = px_size * nw as f64;
        let h = px_size * nh as f64;
        Self { px_size, nw, nh, x, y, w, h,
            x0: x - (w - px_size) / 2.0, // x - w / 2.0 + px_size / 2.0
            y0: y + (h - px_size) / 2.0, // y + h / 2.0 - px_size / 2.0
        }
    }
    fn set_px_size(&mut self, px_size: f64) {
        self.px_size = px_size;
        self.w = px_size * self.nw as f64;
        self.h = px_size * self.nh as f64;
        self.x0 = self.x - (self.w - px_size) / 2.0;
        self.y0 = self.y + (self.h - px_size) / 2.0;
    }
    fn set_x(&mut self, x: f64) {
        self.x = x;
        self.x0 = x - (self.w - self.px_size) / 2.0;
    }
    fn set_y(&mut self, y: f64) {
        self.y = y;
        self.y0 = y + (self.h - self.px_size) / 2.0;
    }
    fn set_nw(&mut self, nw: u32) {
        self.nw = nw;
        self.w = self.px_size * nw as f64;
        self.x0 = self.x - (self.w - self.px_size) / 2.0;
    }
    fn set_nh(&mut self, nh: u32) {
        self.nh = nh;
        self.h = self.px_size * nh as f64;
        self.y0 = self.y + (self.h - self.px_size) / 2.0;
    }
    fn x_at(&self, j: u32) -> f64 {
        self.x0 + self.px_size * j as f64
    }
    fn y_at(&self, i: u32) -> f64 {
        self.y0 - self.px_size * i as f64
    }
}

struct RGBA {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl RGBA {
    fn _rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    fn grayscale(value: f32) -> Self {
        Self { r: value, g: value, b: value, a: 1.0 }
    }
}

fn mandelbrot(x: f64, y: f64) -> RGBA {
    let max_i = 500;
    let mut i = 0;
    let mut x2 = x;
    let mut y2 = y;
    let mut xx = x * x;
    let mut yy = y * y;
    while xx + yy < 4.0 && i < max_i {
        y2 = 2.0 * x2 * y2 + y;
        x2 = xx - yy + x;
        xx = x2 * x2;
        yy = y2 * y2;
        i += 1;
    }
    let ratio = i as f32 / max_i as f32;
    // let hsl = Color::hsl((ratio * 360.0).powf(1.5) % 360.0, 1.0, 1.0 - ratio);
    // RGBA::rgb(hsl.r(), hsl.g(), hsl.b())
    RGBA::grayscale(ratio)
}

fn custom_image(width: usize, height: usize) -> (Image, View, WorldFunc) {
    let view = View::new(4.0 / width as f64, width as _, height as _, 0.0, 0.0);

    let image_buf = create_image_core(&view, mandelbrot);

    let image = create_image(width, height, image_buf);
    let wf = WorldFunc { f: mandelbrot };
    (image, view, wf)
}

fn create_image_core(view: &View, f: impl FnMut(f64, f64) -> RGBA) -> Vec<u8> {
    let total_bytes = (view.nw * view.nh * 4) as usize;
    let mut image_content = Vec::with_capacity(total_bytes);
    rasterize(&view, f, &mut image_content);
    unsafe { image_content.set_len(total_bytes) };
    image_content
}

fn rasterize(view: &View, mut f: impl FnMut(f64, f64) -> RGBA, dest: &mut Vec<u8>) {
    let buf = dest.as_mut_ptr();
    for i in 0..view.nh {
        for j in 0..view.nw {
            let index = ((i * view.nw + j) * 4) as usize;
            let x = view.x_at(j as _);
            let y = view.y_at(i as _);
            let color = f(x, y);
            unsafe {
                let pixel = buf.add(index);
                *pixel.add(0) = (color.r * 255.0) as u8;
                *pixel.add(1) = (color.g * 255.0) as u8;
                *pixel.add(2) = (color.b * 255.0) as u8;
                *pixel.add(3) = (color.a * 255.0) as u8;
            }
        }
    }
}

fn create_image(width: usize, height: usize, data: Vec<u8>) -> Image {
    let size = Extent3d { width: width as _, height: height as _, depth_or_array_layers: 1 };
    Image::new(size, TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb)
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_image: ResMut<Assets<Image>>,
    windows: Query<&Window>,
) {
    commands.spawn(Camera2dBundle::default());

    let window = windows.single();
    let (image, view, wf) = custom_image(window.width() as usize, window.height() as usize);
    commands.spawn((
        ImageFractal,
        SpriteBundle {
            texture: assets_image.add(image),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        view,
        wf,
    ));
    commands.spawn((
        FPSText,
        TextBundle {
            style: Style {
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            text: Text::from_sections([
                TextSection::new("FPS: ", TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::TURQUOISE,
                }),
                TextSection::new("Resolution: ", TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::TURQUOISE,
                }),
            ]),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));

}

#[derive(Resource)]
struct FPSTimer(Timer);

fn text_fps_update(
    time: Res<Time>,
    mut timer: ResMut<FPSTimer>,
    diagnostics: Res<Diagnostics>,
    mut q_text: Query<&mut Text, With<FPSText>>,
    q_view: Query<&View, With<ImageFractal>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if let Some(diag_fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps) = diag_fps.smoothed() {
                for mut fps_text in &mut q_text {
                    fps_text.sections[0].value = format!("FPS: {:.2}", fps);
                }
            }
        }
    }
    let view = q_view.single();
    for mut fps_text in &mut q_text {
        fps_text.sections[1].value = format!("\nPosition: {:.3}, {:.3}", view.x, view.y);
    }
}

fn update_image(
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&mut Handle<Image>, &View, &WorldFunc), (With<ImageFractal>, Changed<View>)>
) {
    for (mut h_image, view, wf) in &mut query {
        if let Some(image) = images.get_mut(&mut *h_image) {
            rasterize(view, wf.f, &mut image.data);
        }
    }
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut q_view: Query<&mut View, With<ImageFractal>>,
    mut q_visible: Query<&mut Visibility, With<FPSText>>,
) {
    for mut view in &mut q_view {
        if keyboard_input.pressed(KeyCode::Left) {
            let new_x = view.x - view.px_size;
            view.set_x(new_x);
        }
        if keyboard_input.pressed(KeyCode::Right) {
            let new_x = view.x + view.px_size;
            view.set_x(new_x);
        }
        if keyboard_input.pressed(KeyCode::Down) {
            let new_y = view.y - view.px_size;
            view.set_y(new_y);
        }
        if keyboard_input.pressed(KeyCode::Up) {
            let new_y = view.y + view.px_size;
            view.set_y(new_y);
        }
        if keyboard_input.pressed(KeyCode::Z) {
            let new_px_size = view.px_size * 0.99;
            view.set_px_size(new_px_size);
        }
        if keyboard_input.pressed(KeyCode::X) {
            let new_px_size = view.px_size / 0.99;
            view.set_px_size(new_px_size);
        }
    }
    for mut vis in &mut q_visible {
        if keyboard_input.just_pressed(KeyCode::C) {
            *vis = match *vis {
                Visibility::Hidden => Visibility::Inherited,
                Visibility::Visible => Visibility::Hidden,
                Visibility::Inherited => Visibility::Hidden,
            };
        }

    }
}

fn on_resize_system(
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&mut Handle<Image>, &mut View, &WorldFunc), With<ImageFractal>>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for (mut h_image, mut view, _wf) in &mut query {
        if let Some(image) = images.get_mut(&mut *h_image) {
            for e in resize_reader.iter() {
                let width = e.width as u32;
                let height = e.height as u32;
                view.set_nw(width);
                view.set_nh(height);
                let size = Extent3d { width, height, depth_or_array_layers: 1 };
                image.resize(size);
            }
        }
    }
}

pub struct MandelbrotPlugin;

impl Plugin for MandelbrotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(FPSTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_startup_system(setup)
            .add_system(text_fps_update)
            .add_system(keyboard_input_system)
            .add_system(on_resize_system)
            .add_system(update_image);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mandelbrot set".to_owned(),
                resolution: WindowResolution::new(400.0, 300.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MandelbrotPlugin)
        .run();
}
