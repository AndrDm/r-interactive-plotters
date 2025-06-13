use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotPoint, Text, PlotUi, Polygon};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Продвинутые аннотации и указатели",
        options,
        Box::new(|_cc| Ok(Box::new(AdvancedAnnotationsApp::default()))),
    )
}

#[derive(Default)]
struct AdvancedAnnotationsApp {
    // Основные данные для графика
    main_data: Vec<[f64; 2]>,
    // Вертикальные указатели
    vertical_markers: Vec<f64>,
    // Горизонтальные указатели
    horizontal_markers: Vec<f64>,
    // Текстовые аннотации с позициями
    text_annotations: Vec<(PlotPoint, String)>,
    // Выделенные области (прямоугольники)
    selection_areas: Vec<(PlotPoint, PlotPoint)>, // (левый_верхний, правый_нижний)
    // Текущее состояние для создания области
    area_start: Option<PlotPoint>,
    // Режимы взаимодействия
    marker_mode: MarkerMode,
    // Последняя позиция мыши
    last_mouse_pos: Option<PlotPoint>,
    // Счетчик для аннотаций
    annotation_counter: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MarkerMode {
    Navigate,       // Обычная навигация по графику
    VerticalLine,   // Добавление вертикальных линий
    HorizontalLine, // Добавление горизонтальных линий
    TextAnnotation, // Добавление текстовых аннотаций
    AreaSelection,  // Выделение областей
}

impl Default for MarkerMode {
    fn default() -> Self {
        MarkerMode::Navigate
    }
}

impl AdvancedAnnotationsApp {
    fn generate_sample_data(&mut self) {
        // Генерируем более сложные данные: синус + косинус + шум
        self.main_data = (0..=200)
            .map(|i| {
                let x = i as f64 * 0.05;
                let y = (x * 0.7).sin() * 1.5 + (x * 1.3).cos() * 0.8 + (x * 3.0).sin() * 0.3;
                [x, y]
            })
            .collect();
    }

    fn draw_vertical_markers(&self, plot_ui: &mut PlotUi) {
        for &x_pos in &self.vertical_markers {
            let bounds = plot_ui.plot_bounds();
            let vertical_line = Line::new(PlotPoints::from(vec![
                [x_pos, bounds.min()[1]],
                [x_pos, bounds.max()[1]]
            ]))
            .color(egui::Color32::from_rgb(255, 165, 0)) // Оранжевый
            .width(2.0)
            .style(egui_plot::LineStyle::Dashed { length: 10.0 });
            plot_ui.line(vertical_line);

            // Добавляем метку
            plot_ui.text(
                Text::new(PlotPoint::new(x_pos, bounds.max()[1] * 0.9), format!("V: {:.1}", x_pos))
                    .color(egui::Color32::DARK_RED)
                    .anchor(egui::Align2::CENTER_BOTTOM)
            );
        }
    }

    fn draw_horizontal_markers(&self, plot_ui: &mut PlotUi) {
        for &y_pos in &self.horizontal_markers {
            let bounds = plot_ui.plot_bounds();
            let horizontal_line = Line::new(PlotPoints::from(vec![
                [bounds.min()[0], y_pos],
                [bounds.max()[0], y_pos]
            ]))
            .color(egui::Color32::from_rgb(0, 255, 127)) // Весенне-зеленый
            .width(2.0)
            .style(egui_plot::LineStyle::Dashed { length: 8.0 });
            plot_ui.line(horizontal_line);

            // Добавляем метку
            plot_ui.text(
                Text::new(PlotPoint::new(bounds.min()[0] * 1.1, y_pos), format!("H: {:.1}", y_pos))
                    .color(egui::Color32::DARK_GREEN)
                    .anchor(egui::Align2::LEFT_CENTER)
            );
        }
    }

    fn draw_text_annotations(&self, plot_ui: &mut PlotUi) {
        for (pos, text) in &self.text_annotations {
            plot_ui.text(
                Text::new(*pos, text)
                    .color(egui::Color32::WHITE)
                    .anchor(egui::Align2::CENTER_CENTER)
                    .highlight(true)
            );
        }
    }

    fn draw_selection_areas(&self, plot_ui: &mut PlotUi) {
        for (start, end) in &self.selection_areas {
            // Создаем прямоугольник
            let points = vec![
                [start.x, start.y],
                [end.x, start.y],
                [end.x, end.y],
                [start.x, end.y],
            ];
            
            let polygon = Polygon::new(PlotPoints::from(points))
                .fill_color(egui::Color32::from_rgba_unmultiplied(100, 200, 255, 50))
                .stroke(egui::Stroke::new(2.0, egui::Color32::BLUE))
                .name(&format!("Область ({:.1},{:.1})-({:.1},{:.1})", start.x, start.y, end.x, end.y));
            plot_ui.polygon(polygon);
        }
    }

    fn draw_preview_area(&self, plot_ui: &mut PlotUi) {
        if let (Some(start), Some(current)) = (self.area_start, self.last_mouse_pos) {
            let points = vec![
                [start.x, start.y],
                [current.x, start.y],
                [current.x, current.y],
                [start.x, current.y],
            ];
            
            let preview_polygon = Polygon::new(PlotPoints::from(points))
                .fill_color(egui::Color32::from_rgba_unmultiplied(255, 255, 0, 30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::YELLOW))
                .name("Предпросмотр области");
            plot_ui.polygon(preview_polygon);
        }
    }
}

impl eframe::App for AdvancedAnnotationsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Генерируем данные при первом запуске
        if self.main_data.is_empty() {
            self.generate_sample_data();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Продвинутые аннотации и указатели");
            
            // Панель управления режимами
            ui.horizontal(|ui| {
                ui.label("Режим:");
                ui.selectable_value(&mut self.marker_mode, MarkerMode::Navigate, "📋 Навигация");
                ui.selectable_value(&mut self.marker_mode, MarkerMode::VerticalLine, "📏 Вертикаль");
                ui.selectable_value(&mut self.marker_mode, MarkerMode::HorizontalLine, "📐 Горизонталь");
                ui.selectable_value(&mut self.marker_mode, MarkerMode::TextAnnotation, "📝 Текст");
                ui.selectable_value(&mut self.marker_mode, MarkerMode::AreaSelection, "🔲 Область");
            });

            ui.horizontal(|ui| {
                if ui.button("🗑 Очистить указатели").clicked() {
                    self.vertical_markers.clear();
                    self.horizontal_markers.clear();
                }
                if ui.button("🗑 Очистить аннотации").clicked() {
                    self.text_annotations.clear();
                }
                if ui.button("🗑 Очистить области").clicked() {
                    self.selection_areas.clear();
                    self.area_start = None;
                }
                if let Some(pos) = self.last_mouse_pos {
                    ui.label(format!("📍 ({:.2}, {:.2})", pos.x, pos.y));
                }
            });

            ui.separator();

            // Основной график
            let _plot_response = Plot::new("advanced_plot")
                .view_aspect(2.0)
                .allow_zoom(true)
                .allow_drag(self.marker_mode == MarkerMode::Navigate)
                .allow_scroll(true)
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    // Обновляем позицию мыши
                    if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        self.last_mouse_pos = Some(pointer_pos);
                    }

                    // Рисуем основную линию
                    let main_line = Line::new(PlotPoints::from(self.main_data.clone()))
                        .color(egui::Color32::BLUE)
                        .width(2.0)
                        .name("Основной сигнал");
                    plot_ui.line(main_line);

                    // Рисуем все аннотации
                    self.draw_vertical_markers(plot_ui);
                    self.draw_horizontal_markers(plot_ui);
                    self.draw_text_annotations(plot_ui);
                    self.draw_selection_areas(plot_ui);
                    self.draw_preview_area(plot_ui);

                    // Обработка кликов в зависимости от режима
                    if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        match self.marker_mode {
                            MarkerMode::VerticalLine => {
                                if plot_ui.response().clicked() {
                                    self.vertical_markers.push(pointer_pos.x);
                                }
                                // Показываем предпросмотр вертикальной линии
                                let bounds = plot_ui.plot_bounds();
                                let preview_line = Line::new(PlotPoints::from(vec![
                                    [pointer_pos.x, bounds.min()[1]],
                                    [pointer_pos.x, bounds.max()[1]]
                                ]))
                                .color(egui::Color32::from_rgba_unmultiplied(255, 165, 0, 128))
                                .width(1.0);
                                plot_ui.line(preview_line);
                            },
                            MarkerMode::HorizontalLine => {
                                if plot_ui.response().clicked() {
                                    self.horizontal_markers.push(pointer_pos.y);
                                }
                                // Показываем предпросмотр горизонтальной линии
                                let bounds = plot_ui.plot_bounds();
                                let preview_line = Line::new(PlotPoints::from(vec![
                                    [bounds.min()[0], pointer_pos.y],
                                    [bounds.max()[0], pointer_pos.y]
                                ]))
                                .color(egui::Color32::from_rgba_unmultiplied(0, 255, 127, 128))
                                .width(1.0);
                                plot_ui.line(preview_line);
                            },
                            MarkerMode::TextAnnotation => {
                                if plot_ui.response().clicked() {
                                    self.annotation_counter += 1;
                                    let annotation_text = format!("Точка #{} ({:.1}, {:.1})", 
                                        self.annotation_counter, pointer_pos.x, pointer_pos.y);
                                    self.text_annotations.push((pointer_pos, annotation_text));
                                }
                                // Показываем предпросмотр аннотации
                                plot_ui.text(
                                    Text::new(pointer_pos, "📍 Клик для добавления")
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 128))
                                        .anchor(egui::Align2::CENTER_TOP)
                                );
                            },
                            MarkerMode::AreaSelection => {
                                if plot_ui.response().clicked() {
                                    if self.area_start.is_none() {
                                        // Начинаем выделение области
                                        self.area_start = Some(pointer_pos);
                                    } else {
                                        // Завершаем выделение области
                                        if let Some(start) = self.area_start {
                                            self.selection_areas.push((start, pointer_pos));
                                            self.area_start = None;
                                        }
                                    }
                                }
                                // Отмена выделения по правому клику
                                if plot_ui.response().secondary_clicked() {
                                    self.area_start = None;
                                }
                            },
                            MarkerMode::Navigate => {
                                // В режиме навигации ничего дополнительного не делаем
                            }
                        }
                    }
                });

            // Информационная панель
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("📊 Статистика:");
                    ui.label(format!("• Вертикальных линий: {}", self.vertical_markers.len()));
                    ui.label(format!("• Горизонтальных линий: {}", self.horizontal_markers.len()));
                    ui.label(format!("• Текстовых аннотаций: {}", self.text_annotations.len()));
                    ui.label(format!("• Выделенных областей: {}", self.selection_areas.len()));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("💡 Инструкция:");
                    match self.marker_mode {
                        MarkerMode::Navigate => ui.label("Используйте мышь для навигации по графику"),
                        MarkerMode::VerticalLine => ui.label("Клик для добавления вертикальной линии"),
                        MarkerMode::HorizontalLine => ui.label("Клик для добавления горизонтальной линии"),
                        MarkerMode::TextAnnotation => ui.label("Клик для добавления текстовой аннотации"),
                        MarkerMode::AreaSelection => ui.label("Клик - начать область, второй клик - завершить. ПКМ - отмена"),
                    };
                    ui.label("🔍 Колесо мыши для масштабирования");
                    ui.label("✨ Все элементы автоматически масштабируются!");
                });
            });

            // Детальная информация о созданных элементах
            ui.separator();
            ui.collapsing("🔍 Подробная информация", |ui| {
                if !self.vertical_markers.is_empty() {
                    ui.label("Вертикальные линии:");
                    for (i, &x) in self.vertical_markers.iter().enumerate() {
                        ui.label(format!("  #{}: x = {:.2}", i + 1, x));
                    }
                }
                if !self.horizontal_markers.is_empty() {
                    ui.label("Горизонтальные линии:");
                    for (i, &y) in self.horizontal_markers.iter().enumerate() {
                        ui.label(format!("  #{}: y = {:.2}", i + 1, y));
                    }
                }
                if !self.text_annotations.is_empty() {
                    ui.label("Текстовые аннотации:");
                    for (i, (pos, text)) in self.text_annotations.iter().enumerate() {
                        ui.label(format!("  #{}: ({:.1}, {:.1}) - \"{}\"", i + 1, pos.x, pos.y, text));
                    }
                }
                if !self.selection_areas.is_empty() {
                    ui.label("Выделенные области:");
                    for (i, (start, end)) in self.selection_areas.iter().enumerate() {
                        let width = (end.x - start.x).abs();
                        let height = (end.y - start.y).abs();
                        ui.label(format!("  #{}: ({:.1}, {:.1}) → ({:.1}, {:.1}) [{}×{}]", 
                            i + 1, start.x, start.y, end.x, end.y, width as i32, height as i32));
                    }
                }
            });
        });
    }
}