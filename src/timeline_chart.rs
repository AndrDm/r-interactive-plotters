use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotPoint, Points, PlotUi};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Временная шкала с горизонтальными маркерами",
        options,
        Box::new(|_cc| Ok(Box::new(TimelineApp::default()))),
    )
}

#[derive(Default)]
struct TimelineApp {
    main_data: Vec<[f64; 2]>,
    horizontal_markers: Vec<HorizontalMarker>,
    show_tooltips: bool,
    hovered_marker: Option<usize>,
    last_hover_pos: Option<PlotPoint>,
}

#[derive(Clone)]
struct HorizontalMarker {
    y_value: f64,
    color: egui::Color32,
    label: String,
    start_x: f64,
    end_x: f64,
}

impl TimelineApp {
    fn format_time(hours_float: f64) -> String {
        let hours = hours_float as i32;
        let minutes = ((hours_float - hours as f64) * 60.0) as i32;
        format!("{:02}:{:02}", hours.max(0).min(23), minutes.max(0).min(59))
    }
    
    fn format_time_with_date(hours_float: f64) -> String {
        let base_date = "2024-01-15"; // Базовая дата для примера
        let hours = hours_float as i32;
        let minutes = ((hours_float - hours as f64) * 60.0) as i32;
        let seconds = (((hours_float - hours as f64) * 60.0 - minutes as f64) * 60.0) as i32;
        format!("{} {:02}:{:02}:{:02}", base_date, hours.max(0).min(23), minutes.max(0).min(59), seconds.max(0).min(59))
    }
    
    fn generate_sample_data(&mut self) {
        // Генерируем плавный многочлен на временной шкале
        let time_range = 0.0..=24.0; // 24 часа
        let num_points = 200;
        
        self.main_data = (0..=num_points)
            .map(|i| {
                let t = *time_range.start() + (*time_range.end() - *time_range.start()) * i as f64 / num_points as f64;
                // Плавная функция: комбинация синуса и полинома
                let base = 50.0; // базовый уровень
                let daily_cycle = 30.0 * (t / 24.0 * 2.0 * std::f64::consts::PI).sin();
                let trend = 0.5 * t.powi(2) - 12.0 * t + 150.0;
                let noise = 5.0 * (t * 3.7).sin() * (t * 1.3).cos();
                
                let value: f64 = base + daily_cycle + trend * 0.1 + noise;
                [t, value.max(10.0).min(100.0)] // ограничиваем диапазон
            })
            .collect();

        // Создаем 10 горизонтальных отрезков равномерно распределенных
        self.horizontal_markers.clear();
        let y_min = 20.0;
        let y_max = 90.0;
        let segment_length = 24.0 / 20.0; // 1/20 дня = 1.2 часа
        let total_time = 24.0;
        
        let colors = [
            egui::Color32::RED,
            egui::Color32::GREEN,
            egui::Color32::BLUE,
            egui::Color32::YELLOW,
            egui::Color32::LIGHT_BLUE,
            egui::Color32::from_rgb(255, 165, 0), // Orange
            egui::Color32::from_rgb(128, 0, 128), // Purple
            egui::Color32::from_rgb(255, 20, 147), // Deep pink
            egui::Color32::from_rgb(0, 128, 0), // Dark green
            egui::Color32::from_rgb(139, 69, 19), // Brown
        ];
        
        for i in 0..10 {
            let y_value = y_min + (y_max - y_min) * i as f64 / 9.0;
            
            // Распределяем отрезки равномерно по времени
            let time_step = total_time / 10.0; // делим 24 часа на 10 отрезков
            let segment_center = time_step * i as f64 + time_step / 2.0;
            let start_x = segment_center - segment_length / 2.0;
            let end_x = segment_center + segment_length / 2.0;
            
            // Убеждаемся, что отрезки не выходят за границы
            let start_x = start_x.max(0.0);
            let end_x = end_x.min(24.0);
            
            let marker = HorizontalMarker {
                y_value,
                color: colors[i % colors.len()],
                label: match i {
                    0 => "Критический минимум".to_string(),
                    1 => "Низкий уровень".to_string(),
                    2 => "Ниже нормы".to_string(),
                    3 => "Нижняя граница нормы".to_string(),
                    4 => "Норма (нижняя)".to_string(),
                    5 => "Оптимальный уровень".to_string(),
                    6 => "Норма (верхняя)".to_string(),
                    7 => "Верхняя граница нормы".to_string(),
                    8 => "Повышенный уровень".to_string(),
                    9 => "Критический максимум".to_string(),
                    _ => format!("Маркер {}", i + 1),
                },
                start_x,
                end_x,
            };
            self.horizontal_markers.push(marker);
        }
    }
    
    fn draw_horizontal_markers(&self, plot_ui: &mut PlotUi) {
        for (_i, marker) in self.horizontal_markers.iter().enumerate() {
            // Рисуем горизонтальный отрезок как линию между двумя точками
            let segment_line = Line::new(PlotPoints::from(vec![
                [marker.start_x, marker.y_value],
                [marker.end_x, marker.y_value]
            ]))
                .color(marker.color)
                .width(3.0)
                .name(&marker.label);
            plot_ui.line(segment_line);
            
            // Рисуем маркеры начала и конца отрезка
            let start_point = Points::new(vec![[marker.start_x, marker.y_value]])
                .color(marker.color)
                .radius(6.0)
                .name(&format!("{} (начало)", marker.label));
            plot_ui.points(start_point);
            
            let end_point = Points::new(vec![[marker.end_x, marker.y_value]])
                .color(marker.color)
                .radius(6.0)
                .name(&format!("{} (конец)", marker.label));
            plot_ui.points(end_point);
        }
    }
    
    fn check_marker_hover(&mut self, plot_ui: &PlotUi) {
        self.hovered_marker = None;
        
        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
            self.last_hover_pos = Some(pointer_pos);
            
            // Проверяем наведение на отрезки маркеров
            for (i, marker) in self.horizontal_markers.iter().enumerate() {
                let y_diff = (pointer_pos.y - marker.y_value).abs();
                let x_in_range = pointer_pos.x >= marker.start_x && pointer_pos.x <= marker.end_x;
                
                // Если курсор близко к горизонтальному отрезку (только в пределах отрезка)
                if y_diff < 2.0 && x_in_range {
                    self.hovered_marker = Some(i);
                    break;
                }
                
                // Проверяем наведение на точки начала и конца отрезка
                let start_dist = ((pointer_pos.x - marker.start_x).powi(2) + (pointer_pos.y - marker.y_value).powi(2)).sqrt();
                let end_dist = ((pointer_pos.x - marker.end_x).powi(2) + (pointer_pos.y - marker.y_value).powi(2)).sqrt();
                
                if start_dist < 1.0 || end_dist < 1.0 {
                    self.hovered_marker = Some(i);
                    break;
                }
            }
        }
    }
}

impl eframe::App for TimelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.main_data.is_empty() {
            self.generate_sample_data();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Временная шкала с горизонтальными отрезками");
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_tooltips, "Показывать подсказки");
                if ui.button("Перегенерировать данные").clicked() {
                    self.generate_sample_data();
                }
                
                ui.separator();
                
                if let Some(pos) = self.last_hover_pos {
                    ui.colored_label(egui::Color32::BLUE, "🕐");
                    ui.strong(format!("Время: {}", Self::format_time(pos.x)));
                    ui.separator();
                    ui.colored_label(egui::Color32::DARK_GREEN, "📅");
                    ui.label(format!("Полная дата: {}", Self::format_time_with_date(pos.x)));
                    ui.separator();
                    ui.colored_label(egui::Color32::DARK_BLUE, "📊");
                    ui.label(format!("Значение: {:.2}", pos.y));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "🕐");
                    ui.label("Наведите курсор на график");
                }
            });

            ui.separator();

            // Основной график
            Plot::new("timeline_plot")
                .height(400.0)
                .view_aspect(3.0)
                .allow_zoom(true)
                .allow_drag(true)
                .allow_scroll(true)
                .x_axis_label("Время")
                .y_axis_label("Показатель")
                .x_axis_formatter(|mark, _range| {
                    TimelineApp::format_time(mark.value)
                })
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("{}\nВремя: {}\nЗначение: {:.1}", 
                            name, 
                            TimelineApp::format_time(value.x), 
                            value.y)
                    } else {
                        format!("Время: {}\nЗначение: {:.1}", 
                            TimelineApp::format_time(value.x), 
                            value.y)
                    }
                })
                .show(ui, |plot_ui| {
                    // Проверяем наведение на маркеры
                    self.check_marker_hover(plot_ui);
                    
                    // Рисуем основную кривую
                    let main_line = Line::new(PlotPoints::from(self.main_data.clone()))
                        .color(egui::Color32::from_rgb(70, 130, 180))
                        .width(3.0)
                        .name("Основные данные");
                    plot_ui.line(main_line);
                    
                    // Рисуем горизонтальные маркеры
                    self.draw_horizontal_markers(plot_ui);
                });

            ui.separator();

            // Показываем подсказку для наведенного маркера
            if self.show_tooltips {
                if let Some(marker_idx) = self.hovered_marker {
                    if let Some(marker) = self.horizontal_markers.get(marker_idx) {
                        ui.horizontal(|ui| {
                            ui.colored_label(marker.color, "●");
                            ui.label(format!("Маркер: {}", marker.label));
                            ui.label(format!("Уровень: {:.1}", marker.y_value));
                            ui.label(format!("Время: {} - {}", 
                                Self::format_time(marker.start_x), 
                                Self::format_time(marker.end_x)));
                        });
                    }
                }
            }

            // Легенда маркеров
            ui.collapsing("📋 Легенда маркеров", |ui| {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (i, marker) in self.horizontal_markers.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.colored_label(marker.color, "●");
                            ui.label(format!("{}: {:.1}", marker.label, marker.y_value));
                            ui.separator();
                            ui.small(format!("{} - {}", 
                                Self::format_time(marker.start_x), 
                                Self::format_time(marker.end_x)));
                            if self.hovered_marker == Some(i) {
                                ui.colored_label(egui::Color32::YELLOW, "← НАВЕДЕН");
                            }
                        });
                    }
                });
            });

            // Инструкции
            ui.separator();
            ui.label("Инструкция:");
            ui.label("• Наведите курсор на горизонтальные отрезки или точки для подсказок");
            ui.label("• Каждый отрезок длиной 1.2 часа (1/20 дня)");
            ui.label("• Отрезки равномерно распределены по 24-часовой шкале");
            ui.label("• Ось X показывает время в формате ЧЧ:ММ");
            ui.label("• Время курсора отображается выше в синей строке 🕐");
            ui.label("• Используйте колесо мыши для масштабирования");
            ui.label("• Перетаскивайте график для панорамирования");
            
            // Отладочная информация
            ui.collapsing("🔧 Отладка", |ui| {
                ui.label(format!("Точек данных: {}", self.main_data.len()));
                ui.label(format!("Горизонтальных маркеров: {}", self.horizontal_markers.len()));
                if let Some(idx) = self.hovered_marker {
                    ui.label(format!("Наведен маркер: {}", idx));
                }
            });
        });
    }
}