use eframe::egui;
use std::process::Command;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 600.0])
            .with_resizable(true)
            .with_min_inner_size([500.0, 550.0]),
        ..Default::default()
    };
    eframe::run_native(
        "🦀 Интерактивные графики - Демонстрационные примеры",
        options,
        Box::new(|_cc| Ok(Box::new(DemoLauncherApp::default()))),
    )
}

#[derive(Default)]
struct DemoLauncherApp {
    last_message: String,
    error_message: String,
}

impl DemoLauncherApp {
    fn launch_demo(&mut self, demo_name: &str, description: &str) {
        self.error_message.clear();
        self.last_message = format!("Запускаю {}...", description);
        
        match Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("--bin")
            .arg(demo_name)
            .spawn()
        {
            Ok(_) => {
                self.last_message = format!("✅ {} запущен успешно", description);
            },
            Err(e) => {
                self.error_message = format!("❌ Ошибка запуска: {}", e);
                self.last_message.clear();
            }
        }
    }
}

impl eframe::App for DemoLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Заголовок
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("🦀 Интерактивные графики с Rust");
                ui.label("Демонстрационные примеры возможностей egui_plot");
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(10.0);

            // Описание
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label("🎯 Все примеры демонстрируют ключевую возможность:");
                    ui.label("   Рисование поверх графика с автоматическим масштабированием");
                    ui.add_space(5.0);
                    ui.label("💡 Нажмите на кнопку для запуска соответствующего примера:");
                });
                ui.add_space(10.0);
            });

            ui.add_space(15.0);

            // Кнопки демонстраций
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    let button_width = ui.available_width() - 20.0;
                    
                    // Пример 1
                    if ui.add_sized([button_width, 40.0], egui::Button::new("🖊 1. Базовое рисование линий"))
                        .on_hover_text("Рисование произвольных линий поверх графика с crosshair")
                        .clicked() 
                    {
                        self.launch_demo("basic_interactive", "Базовое рисование линий");
                    }

                    ui.add_space(8.0);

                    // Пример 2
                    if ui.add_sized([button_width, 40.0], egui::Button::new("📝 2. Продвинутые аннотации"))
                        .on_hover_text("Указатели, текстовые аннотации, выделение областей")
                        .clicked() 
                    {
                        self.launch_demo("advanced_annotations", "Продвинутые аннотации");
                    }

                    ui.add_space(8.0);

                    // Пример 3
                    if ui.add_sized([button_width, 40.0], egui::Button::new("📏 3. Интерактивные измерения"))
                        .on_hover_text("Crosshair, измерительная линейка, snap к данным")
                        .clicked() 
                    {
                        self.launch_demo("interactive_measurements", "Интерактивные измерения");
                    }

                    ui.add_space(8.0);

                    // Пример 4
                    if ui.add_sized([button_width, 40.0], egui::Button::new("🔄 4. Система координат"))
                        .on_hover_text("Преобразования координат, события мыши, демо точки")
                        .clicked() 
                    {
                        self.launch_demo("coordinate_system", "Система координат");
                    }
                });
                ui.add_space(10.0);
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(8.0);

            // Статус и сообщения
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    if !self.last_message.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("📢 Статус:");
                            ui.label(&self.last_message);
                        });
                    }
                    
                    if !self.error_message.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("⚠️ Ошибка:");
                            ui.colored_label(egui::Color32::RED, &self.error_message);
                        });
                    }

                    if self.last_message.is_empty() && self.error_message.is_empty() {
                        ui.label("💡 Выберите пример для запуска");
                    }
                });
                ui.add_space(10.0);
            });

            ui.add_space(8.0);

            // Содержимое в скроллируемой области
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Дополнительная информация
                ui.collapsing("ℹ️ Подробная информация о примерах", |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.label("🖊 Базовое рисование:");
                            ui.label("   • Режим рисования линий поверх графика");
                            ui.label("   • Crosshair в реальном времени");
                            ui.label("   • ЛКМ - добавить точку, ПКМ - завершить линию");
                            
                            ui.add_space(6.0);
                            
                            ui.label("📝 Продвинутые аннотации:");
                            ui.label("   • 5 режимов: навигация, вертикаль, горизонталь, текст, область");
                            ui.label("   • Предпросмотр элементов в реальном времени");
                            ui.label("   • Автоматическая нумерация и позиционирование");
                            
                            ui.add_space(6.0);
                            
                            ui.label("📏 Интерактивные измерения:");
                            ui.label("   • Crosshair с координатами");
                            ui.label("   • Точные измерения расстояний");
                            ui.label("   • Snap к точкам данных (магнитная привязка)");
                            
                            ui.add_space(6.0);
                            
                            ui.label("🔄 Система координат:");
                            ui.label("   • Демонстрация преобразований координат");
                            ui.label("   • Отслеживание событий мыши");
                            ui.label("   • Перетаскиваемые демонстрационные элементы");
                        });
                        ui.add_space(10.0);
                    });
                });

                ui.add_space(5.0);

                // Инструкции по сборке
                ui.collapsing("🔧 Альтернативные способы запуска", |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.label("Из командной строки:");
                            ui.code("cargo run --bin basic_interactive");
                            ui.code("cargo run --bin advanced_annotations");
                            ui.code("cargo run --bin interactive_measurements");
                            ui.code("cargo run --bin coordinate_system");
                            
                            ui.add_space(6.0);
                            ui.label("Сборка всех примеров:");
                            ui.code("cargo build --release");
                            
                            ui.add_space(6.0);
                            ui.label("Запуск готовых бинарников:");
                            ui.code("./target/release/basic_interactive");
                            ui.code("./target/release/advanced_annotations");
                            ui.code("./target/release/interactive_measurements");
                            ui.code("./target/release/coordinate_system");
                        });
                        ui.add_space(10.0);
                    });
                });
                
                ui.add_space(10.0);
            });
        });
    }
}