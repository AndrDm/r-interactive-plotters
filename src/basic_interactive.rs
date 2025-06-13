use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotPoint};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Базовый интерактивный график с рисованием линий",
        options,
        Box::new(|_cc| Ok(Box::new(InteractiveApp::default()))),
    )
}

#[derive(Default)]
struct InteractiveApp {
    // Основные данные для графика
    main_data: Vec<[f64; 2]>,
    // Нарисованные пользователем линии (каждая линия - вектор точек)
    user_lines: Vec<Vec<[f64; 2]>>,
    // Текущая рисуемая линия
    current_line: Vec<[f64; 2]>,
    // Режим рисования
    drawing_mode: bool,
    // Последняя позиция мыши для отладки
    last_mouse_pos: Option<PlotPoint>,
}

impl InteractiveApp {
    fn generate_sample_data(&mut self) {
        // Генерируем синусоиду как основные данные
        self.main_data = (0..=100)
            .map(|i| {
                let x = i as f64 * 0.1;
                [x, (x * 0.5).sin() * 2.0]
            })
            .collect();
    }
}

impl eframe::App for InteractiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Генерируем данные при первом запуске
        if self.main_data.is_empty() {
            self.generate_sample_data();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Интерактивный график с рисованием поверх");
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.drawing_mode, "Режим рисования");
                if ui.button("Очистить линии").clicked() {
                    self.user_lines.clear();
                    self.current_line.clear();
                }
                if let Some(pos) = self.last_mouse_pos {
                    ui.label(format!("Позиция мыши: ({:.2}, {:.2})", pos.x, pos.y));
                }
            });

            ui.separator();

            // Основной график с интерактивностью
            let plot_response = Plot::new("interactive_plot")
                .view_aspect(2.0)
                .allow_zoom(true)
                .allow_drag(true)
                .allow_scroll(true)
                .show(ui, |plot_ui| {
                    // Получаем позицию мыши в координатах графика
                    if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        self.last_mouse_pos = Some(pointer_pos);
                    }

                    // Рисуем основную линию (синусоиду)
                    let main_line = Line::new(PlotPoints::from(self.main_data.clone()))
                        .color(egui::Color32::BLUE)
                        .width(2.0)
                        .name("Основной график");
                    plot_ui.line(main_line);

                    // Рисуем завершенные пользовательские линии
                    for (i, line_points) in self.user_lines.iter().enumerate() {
                        if line_points.len() >= 2 {
                            let user_line = Line::new(PlotPoints::from(line_points.clone()))
                                .color(egui::Color32::RED)
                                .width(3.0)
                                .name(&format!("Линия {}", i + 1));
                            plot_ui.line(user_line);
                        }
                    }

                    // Рисуем текущую рисуемую линию
                    if self.current_line.len() >= 2 {
                        let current_line = Line::new(PlotPoints::from(self.current_line.clone()))
                            .color(egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128))
                            .width(2.0)
                            .name("Текущая линия");
                        plot_ui.line(current_line);
                    }

                    // Обработка кликов мыши в режиме рисования
                    if self.drawing_mode {
                        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                            // При нажатии левой кнопки мыши
                            if plot_ui.response().clicked() {
                                // Начинаем новую линию или добавляем точку к текущей
                                self.current_line.push([pointer_pos.x, pointer_pos.y]);
                            }
                            
                            // При нажатии правой кнопки - завершаем текущую линию
                            if plot_ui.response().secondary_clicked() {
                                if !self.current_line.is_empty() {
                                    self.user_lines.push(self.current_line.clone());
                                    self.current_line.clear();
                                }
                            }
                        }
                    }

                    // Показываем перекрестие в позиции мыши для режима рисования
                    if self.drawing_mode {
                        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                            // Вертикальная линия
                            let vertical_line = Line::new(PlotPoints::from(vec![
                                [pointer_pos.x, plot_ui.plot_bounds().min()[1]],
                                [pointer_pos.x, plot_ui.plot_bounds().max()[1]]
                            ]))
                            .color(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 100))
                            .width(1.0);
                            plot_ui.line(vertical_line);

                            // Горизонтальная линия
                            let horizontal_line = Line::new(PlotPoints::from(vec![
                                [plot_ui.plot_bounds().min()[0], pointer_pos.y],
                                [plot_ui.plot_bounds().max()[0], pointer_pos.y]
                            ]))
                            .color(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 100))
                            .width(1.0);
                            plot_ui.line(horizontal_line);
                        }
                    }
                });

            // Инструкции для пользователя
            ui.separator();
            ui.label("Инструкция:");
            ui.label("• Включите 'Режим рисования'");
            ui.label("• Левый клик - добавить точку к линии");
            ui.label("• Правый клик - завершить текущую линию");
            ui.label("• Используйте колесо мыши для масштабирования");
            ui.label("• Перетаскивайте график для панорамирования");
            ui.label("• Все нарисованные линии автоматически масштабируются!");

            // Отладочная информация
            ui.separator();
            ui.collapsing("Отладка", |ui| {
                ui.label(format!("Текущая линия: {} точек", self.current_line.len()));
                ui.label(format!("Завершенных линий: {}", self.user_lines.len()));
                if let transform = plot_response.transform {
                    let bounds = transform.bounds();
                    ui.label(format!("Границы графика: {:?}", bounds));
                }
            });
        });
    }
}