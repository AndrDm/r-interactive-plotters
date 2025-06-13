use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotPoint, Text, Points};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Система координат и события мыши",
        options,
        Box::new(|_cc| Ok(Box::new(CoordinateSystemApp::default()))),
    )
}

struct CoordinateSystemApp {
    // Основные данные для графика
    main_data: Vec<[f64; 2]>,
    // Информация о событиях мыши
    mouse_events: Vec<MouseEvent>,
    max_events: usize,
    // Система координат
    plot_bounds: Option<egui_plot::PlotBounds>,
    screen_rect: Option<egui::Rect>,
    transform_info: Option<TransformInfo>,
    // Режим отображения
    show_grid_info: bool,
    show_transform_demo: bool,
    show_event_history: bool,
    // Демонстрационные точки в разных системах координат
    demo_points_plot: Vec<PlotPoint>,
    demo_points_screen: Vec<egui::Pos2>,
    // Интерактивные элементы
    draggable_point: Option<PlotPoint>,
    is_dragging: bool,
    // Масштаб и панорамирование
    zoom_history: Vec<ZoomInfo>,
    current_zoom_level: f64,
}

#[derive(Clone, Debug)]
struct MouseEvent {
    event_type: MouseEventType,
    plot_pos: Option<PlotPoint>,
    screen_pos: Option<egui::Pos2>,
    timestamp: std::time::Instant,
    additional_info: String,
}

#[derive(Clone, Debug)]
enum MouseEventType {
    Click,
    DoubleClick,
    RightClick,
    Drag,
    Hover,
    Scroll,
    Enter,
    Leave,
}

#[derive(Clone, Debug)]
struct TransformInfo {
    plot_to_screen_scale: egui::Vec2,
    screen_to_plot_scale: egui::Vec2,
    plot_offset: egui::Vec2,
    screen_offset: egui::Vec2,
}

#[derive(Clone, Debug)]
struct ZoomInfo {
    bounds: egui_plot::PlotBounds,
    zoom_level: f64,
    center: PlotPoint,
    timestamp: std::time::Instant,
}

impl Default for CoordinateSystemApp {
    fn default() -> Self {
        Self {
            main_data: Vec::new(),
            mouse_events: Vec::new(),
            max_events: 0,
            plot_bounds: None,
            screen_rect: None,
            transform_info: None,
            show_grid_info: false,
            show_transform_demo: false,
            show_event_history: false,
            demo_points_plot: Vec::new(),
            demo_points_screen: Vec::new(),
            draggable_point: None,
            is_dragging: false,
            zoom_history: Vec::new(),
            current_zoom_level: 0.0,
        }
    }
}

impl CoordinateSystemApp {
    fn generate_sample_data(&mut self) {
        // Генерируем спиральные данные для демонстрации координат
        self.main_data = (0..=500)
            .map(|i| {
                let t = i as f64 * 0.02;
                let r = t * 0.1;
                let x = r * (t * 2.0).cos();
                let y = r * (t * 2.0).sin();
                [x, y]
            })
            .collect();

        // Начальные настройки
        self.max_events = 50;
        self.show_grid_info = true;
        self.show_transform_demo = true;
        self.show_event_history = true;
        self.draggable_point = Some(PlotPoint::new(1.0, 1.0));
        self.current_zoom_level = 1.0;

        // Демонстрационные точки
        self.demo_points_plot = vec![
            PlotPoint::new(-2.0, -1.5),
            PlotPoint::new(0.0, 0.0),
            PlotPoint::new(1.5, 2.0),
            PlotPoint::new(-1.0, 2.5),
        ];
    }

    fn add_mouse_event(&mut self, event_type: MouseEventType, plot_pos: Option<PlotPoint>, 
                       screen_pos: Option<egui::Pos2>, additional_info: String) {
        let event = MouseEvent {
            event_type,
            plot_pos,
            screen_pos,
            timestamp: std::time::Instant::now(),
            additional_info,
        };

        self.mouse_events.push(event);
        
        // Ограничиваем количество событий
        if self.mouse_events.len() > self.max_events {
            self.mouse_events.remove(0);
        }
    }

    fn calculate_transform_info(&self, plot_ui: &egui_plot::PlotUi) -> TransformInfo {
        let bounds = plot_ui.plot_bounds();
        let screen_bounds = plot_ui.screen_from_plot(PlotPoint::new(bounds.min()[0], bounds.min()[1])) 
            .distance(plot_ui.screen_from_plot(PlotPoint::new(bounds.max()[0], bounds.max()[1])));
        
        let plot_width = bounds.max()[0] - bounds.min()[0];
        let plot_height = bounds.max()[1] - bounds.min()[1];
        
        TransformInfo {
            plot_to_screen_scale: egui::Vec2::new(
                screen_bounds / plot_width as f32,
                screen_bounds / plot_height as f32
            ),
            screen_to_plot_scale: egui::Vec2::new(
                plot_width as f32 / screen_bounds,
                plot_height as f32 / screen_bounds
            ),
            plot_offset: egui::Vec2::new(bounds.min()[0] as f32, bounds.min()[1] as f32),
            screen_offset: plot_ui.screen_from_plot(PlotPoint::new(bounds.min()[0], bounds.min()[1])).to_vec2(),
        }
    }

    fn draw_coordinate_grid(&self, plot_ui: &mut egui_plot::PlotUi) {
        let bounds = plot_ui.plot_bounds();
        let min_x = bounds.min()[0];
        let max_x = bounds.max()[0];
        let min_y = bounds.min()[1];
        let max_y = bounds.max()[1];

        // Основные оси
        if min_x <= 0.0 && max_x >= 0.0 {
            let y_axis = Line::new(PlotPoints::from(vec![[0.0, min_y], [0.0, max_y]]))
                .color(egui::Color32::from_rgb(100, 100, 100))
                .width(2.0)
                .name("Y-ось");
            plot_ui.line(y_axis);
        }

        if min_y <= 0.0 && max_y >= 0.0 {
            let x_axis = Line::new(PlotPoints::from(vec![[min_x, 0.0], [max_x, 0.0]]))
                .color(egui::Color32::from_rgb(100, 100, 100))
                .width(2.0)
                .name("X-ось");
            plot_ui.line(x_axis);
        }

        // Сетка
        let grid_step_x = (max_x - min_x) / 10.0;
        let grid_step_y = (max_y - min_y) / 10.0;

        // Вертикальные линии сетки
        let mut x = (min_x / grid_step_x).ceil() * grid_step_x;
        while x <= max_x {
            if (x - 0.0).abs() > grid_step_x * 0.1 {
                let grid_line = Line::new(PlotPoints::from(vec![[x, min_y], [x, max_y]]))
                    .color(egui::Color32::from_rgba_unmultiplied(80, 80, 80, 100))
                    .width(1.0);
                plot_ui.line(grid_line);
            }
            x += grid_step_x;
        }

        // Горизонтальные линии сетки
        let mut y = (min_y / grid_step_y).ceil() * grid_step_y;
        while y <= max_y {
            if (y - 0.0).abs() > grid_step_y * 0.1 {
                let grid_line = Line::new(PlotPoints::from(vec![[min_x, y], [max_x, y]]))
                    .color(egui::Color32::from_rgba_unmultiplied(80, 80, 80, 100))
                    .width(1.0);
                plot_ui.line(grid_line);
            }
            y += grid_step_y;
        }
    }

    fn draw_demo_points(&mut self, plot_ui: &mut egui_plot::PlotUi) {
        // Обновляем экранные координаты демонстрационных точек
        self.demo_points_screen = self.demo_points_plot.iter()
            .map(|&p| plot_ui.screen_from_plot(PlotPoint::new(p.x, p.y)))
            .collect();

        // Рисуем демонстрационные точки
        for (i, &point) in self.demo_points_plot.iter().enumerate() {
            let screen_pos = plot_ui.screen_from_plot(point);
            
            // Точка
            let demo_point = Points::new(vec![[point.x, point.y]])
                .color(egui::Color32::from_rgb(255, 100 + i as u8 * 40, 255))
                .radius(6.0)
                .shape(egui_plot::MarkerShape::Circle);
            plot_ui.points(demo_point);

            // Подпись с координатами
            plot_ui.text(
                Text::new(
                    PlotPoint::new(point.x + 0.1, point.y + 0.1),
                    format!("P{}: ({:.1}, {:.1})\nScreen: ({:.0}, {:.0})", 
                        i + 1, point.x, point.y, screen_pos.x, screen_pos.y)
                )
                .color(egui::Color32::WHITE)
                .anchor(egui::Align2::LEFT_BOTTOM)
                .highlight(true)
            );
        }

        // Перетаскиваемая точка
        if let Some(draggable_point) = self.draggable_point {
            let _draggable_screen_pos = plot_ui.screen_from_plot(draggable_point);
            let draggable_point_visual = Points::new(vec![[draggable_point.x, draggable_point.y]])
                .color(egui::Color32::GOLD)
                .radius(8.0)
                .shape(egui_plot::MarkerShape::Diamond);
            plot_ui.points(draggable_point_visual);

            plot_ui.text(
                Text::new(
                    PlotPoint::new(draggable_point.x, draggable_point.y + 0.3),
                    format!("Перетаскиваемая\n({:.2}, {:.2})", draggable_point.x, draggable_point.y)
                )
                .color(egui::Color32::YELLOW)
                .anchor(egui::Align2::CENTER_BOTTOM)
                .highlight(true)
            );
        }
    }

    fn format_event_type(event_type: &MouseEventType) -> &'static str {
        match event_type {
            MouseEventType::Click => "🖱 Клик",
            MouseEventType::DoubleClick => "🖱🖱 Двойной клик",
            MouseEventType::RightClick => "🖱 ПКМ",
            MouseEventType::Drag => "↔ Перетаскивание",
            MouseEventType::Hover => "👆 Наведение",
            MouseEventType::Scroll => "🔄 Прокрутка",
            MouseEventType::Enter => "→ Вход",
            MouseEventType::Leave => "← Выход",
        }
    }
}

impl eframe::App for CoordinateSystemApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Генерируем данные при первом запуске
        if self.main_data.is_empty() {
            self.generate_sample_data();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Система координат и события мыши");
            
            // Панель управления
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_grid_info, "📐 Сетка координат");
                ui.checkbox(&mut self.show_transform_demo, "🔄 Демо преобразований");
                ui.checkbox(&mut self.show_event_history, "📋 История событий");
                
                ui.separator();
                
                if ui.button("🗑 Очистить события").clicked() {
                    self.mouse_events.clear();
                }
                
                ui.label(format!("События: {}/{}", self.mouse_events.len(), self.max_events));
            });

            ui.separator();

            // Основной график
            let _plot_response = Plot::new("coordinate_demo")
                .view_aspect(1.0)
                .allow_zoom(true)
                .allow_drag(true)
                .allow_scroll(true)
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    // Сохраняем информацию о границах и трансформации
                    self.plot_bounds = Some(plot_ui.plot_bounds());
                    self.transform_info = Some(self.calculate_transform_info(plot_ui));

                    // Рисуем сетку координат
                    if self.show_grid_info {
                        self.draw_coordinate_grid(plot_ui);
                    }

                    // Рисуем основные данные
                    let main_line = Line::new(PlotPoints::from(self.main_data.clone()))
                        .color(egui::Color32::BLUE)
                        .width(2.0)
                        .name("Спираль");
                    plot_ui.line(main_line);

                    // Демонстрационные точки и преобразования
                    if self.show_transform_demo {
                        self.draw_demo_points(plot_ui);
                    }

                    // Обработка событий мыши
                    let response = plot_ui.response();
                    
                    if let Some(plot_pos) = plot_ui.pointer_coordinate() {
                        let screen_pos = plot_ui.screen_from_plot(plot_pos);

                        // Клики
                        if response.clicked() {
                            self.add_mouse_event(MouseEventType::Click, Some(plot_pos), Some(screen_pos),
                                format!("Левый клик"));
                        }
                        
                        if response.double_clicked() {
                            self.add_mouse_event(MouseEventType::DoubleClick, Some(plot_pos), Some(screen_pos),
                                format!("Двойной клик"));
                        }
                        
                        if response.secondary_clicked() {
                            self.add_mouse_event(MouseEventType::RightClick, Some(plot_pos), Some(screen_pos),
                                format!("Правый клик"));
                        }

                        // Перетаскивание точки
                        if let Some(draggable_point) = self.draggable_point {
                            let distance_to_draggable = ((plot_pos.x - draggable_point.x).powi(2) + 
                                                       (plot_pos.y - draggable_point.y).powi(2)).sqrt();
                            
                            if distance_to_draggable < 0.2 && response.drag_started() {
                                self.is_dragging = true;
                                self.add_mouse_event(MouseEventType::Drag, Some(plot_pos), Some(screen_pos),
                                    format!("Начало перетаскивания точки"));
                            }
                        }
                        
                        if self.is_dragging && response.dragged() {
                            self.draggable_point = Some(plot_pos);
                            self.add_mouse_event(MouseEventType::Drag, Some(plot_pos), Some(screen_pos),
                                format!("Перетаскивание точки"));
                        }
                        
                        if response.drag_stopped() {
                            self.is_dragging = false;
                            self.add_mouse_event(MouseEventType::Drag, Some(plot_pos), Some(screen_pos),
                                format!("Окончание перетаскивания"));
                        }

                        // Прокрутка
                        if response.hovered() {
                            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta);
                            if scroll_delta.length() > 0.0 {
                                self.add_mouse_event(MouseEventType::Scroll, Some(plot_pos), Some(screen_pos),
                                    format!("Прокрутка: ({:.1}, {:.1})", scroll_delta.x, scroll_delta.y));
                            }
                        }
                    }

                    // Вход и выход курсора
                    if response.hovered() && self.mouse_events.last()
                        .map_or(true, |e| !matches!(e.event_type, MouseEventType::Hover)) {
                        self.add_mouse_event(MouseEventType::Enter, 
                            plot_ui.pointer_coordinate(), 
                            plot_ui.pointer_coordinate().map(|p| plot_ui.screen_from_plot(p)),
                            "Курсор вошел в область графика".to_string());
                    }
                });

            // Информационные панели
            ui.separator();
            
            ui.horizontal(|ui| {
                // Информация о границах и трансформации
                ui.vertical(|ui| {
                    ui.label("🔍 Информация о графике:");
                    if let Some(bounds) = &self.plot_bounds {
                        ui.label(format!("X: [{:.3}, {:.3}]", bounds.min()[0], bounds.max()[0]));
                        ui.label(format!("Y: [{:.3}, {:.3}]", bounds.min()[1], bounds.max()[1]));
                        let width = bounds.max()[0] - bounds.min()[0];
                        let height = bounds.max()[1] - bounds.min()[1];
                        ui.label(format!("Размер: {:.3} × {:.3}", width, height));
                        
                        // Уровень масштабирования
                        let base_width = 10.0; // базовая ширина
                        self.current_zoom_level = base_width / width;
                        ui.label(format!("Масштаб: {:.2}x", self.current_zoom_level));
                    }
                });

                ui.separator();

                // Информация о преобразованиях
                if let Some(transform) = &self.transform_info {
                    ui.vertical(|ui| {
                        ui.label("🔄 Преобразования:");
                        ui.label(format!("Plot→Screen: ({:.1}, {:.1})", 
                            transform.plot_to_screen_scale.x, transform.plot_to_screen_scale.y));
                        ui.label(format!("Screen→Plot: ({:.4}, {:.4})", 
                            transform.screen_to_plot_scale.x, transform.screen_to_plot_scale.y));
                    });
                }

                ui.separator();

                // Текущая позиция мыши
                ui.vertical(|ui| {
                    ui.label("📍 Текущая позиция:");
                    if let Some(last_event) = self.mouse_events.last() {
                        if let Some(plot_pos) = last_event.plot_pos {
                            ui.label(format!("Plot: ({:.3}, {:.3})", plot_pos.x, plot_pos.y));
                        }
                        if let Some(screen_pos) = last_event.screen_pos {
                            ui.label(format!("Screen: ({:.0}, {:.0})", screen_pos.x, screen_pos.y));
                        }
                    }
                });
            });

            // История событий мыши
            if self.show_event_history && !self.mouse_events.is_empty() {
                ui.separator();
                ui.collapsing("📋 История событий мыши", |ui| {
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (i, event) in self.mouse_events.iter().rev().enumerate() {
                            if i >= 20 { break; } // Показываем только последние 20 событий
                            
                            ui.horizontal(|ui| {
                                ui.label(format!("{:02}:", self.mouse_events.len() - i));
                                ui.label(Self::format_event_type(&event.event_type));
                                
                                if let Some(plot_pos) = event.plot_pos {
                                    ui.label(format!("({:.2}, {:.2})", plot_pos.x, plot_pos.y));
                                }
                                
                                if !event.additional_info.is_empty() {
                                    ui.separator();
                                    ui.label(&event.additional_info);
                                }
                            });
                        }
                    });
                });
            }

            // Инструкции
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("💡 Возможности:");
                    ui.label("• Все виды кликов отслеживаются");
                    ui.label("• Перетаскивание золотой точки");
                    ui.label("• Преобразование координат в реальном времени");
                    ui.label("• Адаптивная сетка координат");
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("🎮 Управление:");
                    ui.label("• ЛКМ, ПКМ, двойной клик");
                    ui.label("• Перетаскивание для панорамирования");
                    ui.label("• Колесо мыши для масштабирования");
                    ui.label("• Все координаты пересчитываются автоматически");
                });
            });
        });
    }
}