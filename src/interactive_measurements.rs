use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotPoint, Text, Points};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Интерактивные измерения и crosshair",
        options,
        Box::new(|_cc| Ok(Box::new(MeasurementApp::default()))),
    )
}

#[derive(Default)]
struct MeasurementApp {
    // Основные данные для графика
    main_data: Vec<[f64; 2]>,
    secondary_data: Vec<[f64; 2]>,
    // Crosshair
    show_crosshair: bool,
    crosshair_pos: Option<PlotPoint>,
    // Измерительная линейка
    ruler_points: Vec<PlotPoint>,
    ruler_active: bool,
    // Snap к данным
    snap_to_data: bool,
    snap_distance: f64,
    // Информация о значениях в точке курсора
    show_values: bool,
    // Масштабирование с сохранением пропорций
    lock_aspect: bool,
    // Последние измерения
    measurements: Vec<MeasurementInfo>,
    measurement_counter: usize,
}

#[derive(Clone)]
struct MeasurementInfo {
    id: usize,
    start: PlotPoint,
    end: PlotPoint,
    distance: f64,
    dx: f64,
    dy: f64,
}

impl MeasurementApp {
    fn generate_sample_data(&mut self) {
        // Генерируем основной сигнал - затухающие колебания
        self.main_data = (0..=300)
            .map(|i| {
                let x = i as f64 * 0.03;
                let y = (x * 2.0).sin() * (-x * 0.1).exp();
                [x, y]
            })
            .collect();

        // Генерируем вторичный сигнал - модулированная синусоида
        self.secondary_data = (0..=300)
            .map(|i| {
                let x = i as f64 * 0.03;
                let y = (x * 3.0).cos() * 0.5 * (1.0 + (x * 0.5).sin() * 0.3);
                [x, y]
            })
            .collect();

        // Параметры по умолчанию
        self.show_crosshair = true;
        self.show_values = true;
        self.snap_distance = 0.1;
    }

    fn find_closest_data_point(&self, target: PlotPoint) -> Option<PlotPoint> {
        if !self.snap_to_data {
            return None;
        }

        let mut closest_point = None;
        let mut min_distance = self.snap_distance;

        // Проверяем основные данные
        for point in &self.main_data {
            let distance = ((point[0] - target.x).powi(2) + (point[1] - target.y).powi(2)).sqrt();
            if distance < min_distance {
                min_distance = distance;
                closest_point = Some(PlotPoint::new(point[0], point[1]));
            }
        }

        // Проверяем вторичные данные
        for point in &self.secondary_data {
            let distance = ((point[0] - target.x).powi(2) + (point[1] - target.y).powi(2)).sqrt();
            if distance < min_distance {
                min_distance = distance;
                closest_point = Some(PlotPoint::new(point[0], point[1]));
            }
        }

        closest_point
    }

    fn get_effective_cursor_position(&self) -> Option<PlotPoint> {
        if let Some(cursor_pos) = self.crosshair_pos {
            if let Some(snapped_pos) = self.find_closest_data_point(cursor_pos) {
                Some(snapped_pos)
            } else {
                Some(cursor_pos)
            }
        } else {
            None
        }
    }

    fn draw_crosshair(&self, plot_ui: &mut egui_plot::PlotUi) {
        if !self.show_crosshair {
            return;
        }

        if let Some(pos) = self.get_effective_cursor_position() {
            let bounds = plot_ui.plot_bounds();
            
            // Вертикальная линия crosshair
            let vertical_line = Line::new(PlotPoints::from(vec![
                [pos.x, bounds.min()[1]],
                [pos.x, bounds.max()[1]]
            ]))
            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150))
            .width(1.0)
            .style(egui_plot::LineStyle::Dotted { spacing: 5.0 });
            plot_ui.line(vertical_line);

            // Горизонтальная линия crosshair
            let horizontal_line = Line::new(PlotPoints::from(vec![
                [bounds.min()[0], pos.y],
                [bounds.max()[0], pos.y]
            ]))
            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150))
            .width(1.0)
            .style(egui_plot::LineStyle::Dotted { spacing: 5.0 });
            plot_ui.line(horizontal_line);

            // Центральная точка
            let center_point = Points::new(vec![[pos.x, pos.y]])
                .color(egui::Color32::YELLOW)
                .radius(4.0)
                .shape(egui_plot::MarkerShape::Circle);
            plot_ui.points(center_point);

            // Координаты рядом с курсором
            if self.show_values {
                plot_ui.text(
                    Text::new(
                        PlotPoint::new(pos.x + (bounds.max()[0] - bounds.min()[0]) * 0.02, 
                                      pos.y + (bounds.max()[1] - bounds.min()[1]) * 0.02),
                        format!("({:.3}, {:.3})", pos.x, pos.y)
                    )
                    .color(egui::Color32::WHITE)
                    .anchor(egui::Align2::LEFT_BOTTOM)
                    .highlight(true)
                );
            }
        }
    }

    fn draw_ruler(&self, plot_ui: &mut egui_plot::PlotUi) {
        // Рисуем завершенные измерения
        for measurement in &self.measurements {
            // Линия измерения
            let ruler_line = Line::new(PlotPoints::from(vec![
                [measurement.start.x, measurement.start.y],
                [measurement.end.x, measurement.end.y]
            ]))
            .color(egui::Color32::from_rgb(255, 100, 100))
            .width(3.0)
            .name(&format!("Измерение #{}", measurement.id));
            plot_ui.line(ruler_line);

            // Точки начала и конца
            let start_point = Points::new(vec![[measurement.start.x, measurement.start.y]])
                .color(egui::Color32::RED)
                .radius(6.0)
                .shape(egui_plot::MarkerShape::Square);
            plot_ui.points(start_point);

            let end_point = Points::new(vec![[measurement.end.x, measurement.end.y]])
                .color(egui::Color32::DARK_RED)
                .radius(6.0)
                .shape(egui_plot::MarkerShape::Diamond);
            plot_ui.points(end_point);

            // Информация об измерении в центре линии
            let mid_x = (measurement.start.x + measurement.end.x) / 2.0;
            let mid_y = (measurement.start.y + measurement.end.y) / 2.0;
            
            plot_ui.text(
                Text::new(
                    PlotPoint::new(mid_x, mid_y),
                    format!("#{}: Δ={:.3}\ndx={:.3}, dy={:.3}", 
                        measurement.id, measurement.distance, measurement.dx, measurement.dy)
                )
                .color(egui::Color32::WHITE)
                .anchor(egui::Align2::CENTER_CENTER)
                .highlight(true)
            );
        }

        // Рисуем активное измерение (если есть)
        if self.ruler_active && self.ruler_points.len() == 1 {
            if let Some(current_pos) = self.get_effective_cursor_position() {
                let start = self.ruler_points[0];
                
                // Линия от первой точки до текущей позиции курсора
                let preview_line = Line::new(PlotPoints::from(vec![
                    [start.x, start.y],
                    [current_pos.x, current_pos.y]
                ]))
                .color(egui::Color32::from_rgba_unmultiplied(255, 100, 100, 128))
                .width(2.0)
                .style(egui_plot::LineStyle::Dashed { length: 8.0 });
                plot_ui.line(preview_line);

                // Динамическая информация
                let distance = ((current_pos.x - start.x).powi(2) + (current_pos.y - start.y).powi(2)).sqrt();
                let dx = current_pos.x - start.x;
                let dy = current_pos.y - start.y;

                plot_ui.text(
                    Text::new(
                        current_pos,
                        format!("Расстояние: {:.3}\ndx: {:.3}, dy: {:.3}", distance, dx, dy)
                    )
                    .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200))
                    .anchor(egui::Align2::LEFT_TOP)
                    .highlight(true)
                );
            }
        }
    }

    fn handle_ruler_click(&mut self, pos: PlotPoint) {
        let effective_pos = self.find_closest_data_point(pos).unwrap_or(pos);

        if self.ruler_points.is_empty() {
            // Начинаем новое измерение
            self.ruler_points.push(effective_pos);
            self.ruler_active = true;
        } else if self.ruler_points.len() == 1 {
            // Завершаем измерение
            let start = self.ruler_points[0];
            let end = effective_pos;
            
            let distance = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
            let dx = end.x - start.x;
            let dy = end.y - start.y;

            self.measurement_counter += 1;
            self.measurements.push(MeasurementInfo {
                id: self.measurement_counter,
                start,
                end,
                distance,
                dx,
                dy,
            });

            self.ruler_points.clear();
            self.ruler_active = false;
        }
    }
}

impl eframe::App for MeasurementApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Генерируем данные при первом запуске
        if self.main_data.is_empty() {
            self.generate_sample_data();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Интерактивные измерения и crosshair");
            
            // Панель управления
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_crosshair, "🎯 Crosshair");
                ui.checkbox(&mut self.show_values, "📊 Показать значения");
                ui.checkbox(&mut self.snap_to_data, "🧲 Snap к данным");
                ui.checkbox(&mut self.lock_aspect, "🔒 Сохранять пропорции");
                
                ui.separator();
                
                if ui.button("📏 Новое измерение").clicked() {
                    self.ruler_points.clear();
                    self.ruler_active = false;
                }
                
                if ui.button("🗑 Очистить все").clicked() {
                    self.measurements.clear();
                    self.ruler_points.clear();
                    self.ruler_active = false;
                    self.measurement_counter = 0;
                }
            });

            // Настройки snap
            if self.snap_to_data {
                ui.horizontal(|ui| {
                    ui.label("Радиус snap:");
                    ui.add(egui::Slider::new(&mut self.snap_distance, 0.01..=0.5).text(""));
                });
            }

            ui.separator();

            // Основной график
            let _plot_response = Plot::new("measurement_plot")
                .view_aspect(if self.lock_aspect { 2.5 } else { -1.0 })
                .allow_zoom(true)
                .allow_drag(true)
                .allow_scroll(true)
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    // Обновляем позицию crosshair
                    if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        self.crosshair_pos = Some(pointer_pos);
                    }

                    // Рисуем основные данные
                    let main_line = Line::new(PlotPoints::from(self.main_data.clone()))
                        .color(egui::Color32::BLUE)
                        .width(2.0)
                        .name("Основной сигнал");
                    plot_ui.line(main_line);

                    let secondary_line = Line::new(PlotPoints::from(self.secondary_data.clone()))
                        .color(egui::Color32::GREEN)
                        .width(1.5)
                        .name("Вторичный сигнал");
                    plot_ui.line(secondary_line);

                    // Рисуем crosshair
                    self.draw_crosshair(plot_ui);

                    // Рисуем измерения
                    self.draw_ruler(plot_ui);

                    // Обработка кликов для измерений
                    if plot_ui.response().clicked() {
                        if let Some(pos) = plot_ui.pointer_coordinate() {
                            self.handle_ruler_click(pos);
                        }
                    }

                    // Отмена текущего измерения по правому клику
                    if plot_ui.response().secondary_clicked() {
                        self.ruler_points.clear();
                        self.ruler_active = false;
                    }
                });

            // Информационная панель
            ui.separator();
            
            ui.horizontal(|ui| {
                // Текущая информация
                ui.vertical(|ui| {
                    ui.label("📍 Текущая позиция:");
                    if let Some(pos) = self.get_effective_cursor_position() {
                        ui.label(format!("  X: {:.4}", pos.x));
                        ui.label(format!("  Y: {:.4}", pos.y));
                        
                        if self.snap_to_data {
                            if self.find_closest_data_point(self.crosshair_pos.unwrap_or(PlotPoint::new(0.0, 0.0))).is_some() {
                                ui.label("  🧲 Привязан к данным");
                            }
                        }
                    } else {
                        ui.label("  Курсор вне графика");
                    }
                });

                ui.separator();

                // Статус измерений
                ui.vertical(|ui| {
                    ui.label("📏 Измерения:");
                    ui.label(format!("  Завершено: {}", self.measurements.len()));
                    if self.ruler_active {
                        ui.label("  🔄 Активное измерение...");
                    }
                    if !self.ruler_points.is_empty() {
                        ui.label("  Клик для завершения");
                    }
                });

                ui.separator();

                // Инструкции
                ui.vertical(|ui| {
                    ui.label("💡 Управление:");
                    ui.label("  ЛКМ - точка измерения");
                    ui.label("  ПКМ - отменить измерение");
                    ui.label("  🔍 Колесо - масштаб");
                    ui.label("  🧲 Snap к ближайшим точкам");
                });
            });

            // Детальная информация об измерениях
            if !self.measurements.is_empty() {
                ui.separator();
                ui.collapsing("📋 Подробная информация об измерениях", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for measurement in &self.measurements {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{:02}:", measurement.id));
                                ui.label(format!("({:.3}, {:.3}) → ({:.3}, {:.3})", 
                                    measurement.start.x, measurement.start.y,
                                    measurement.end.x, measurement.end.y));
                                ui.separator();
                                ui.label(format!("Δ: {:.4}", measurement.distance));
                                ui.label(format!("dx: {:.4}", measurement.dx));
                                ui.label(format!("dy: {:.4}", measurement.dy));
                            });
                        }
                    });
                });
            }
        });
    }
}