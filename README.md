## Экспортер метрик GPU в Prometheus

### Сборка и запуск:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://gitflic.ru/project/kedess/gpu-prometheus-exporter.git
cd gpu-prometheus-exporter
cargo build --release
./target/release/gpu-prometheus-exporter
```

### Установка в виде сервиса:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://gitflic.ru/project/kedess/gpu-prometheus-exporter.git
cd gpu-prometheus-exporter
cargo build --release
sudo cp ./target/release/gpu-prometheus-exporter /usr/bin
sudo cp gpu-prometheus-exporter.service /etc/systemd/system
sudo systemctl daemon-reload
sudo systemctl enable --now gpu-prometheus-exporter
```

### Запуск в контейнере docker:
```bash
docker run --rm -p 9835:9835 -v /usr/bin/nvidia-smi:/usr/bin/nvidia-smi --device /dev/nvidiactl:/dev/nvidiactl --device /dev/nvidia0:/dev/nvidia0 -v /usr/lib/libnvidia-ml.so:/usr/lib/libnvidia-ml.so -v /usr/lib/libnvidia-ml.so.1:/usr/lib/libnvidia-ml.so.1 kedess/gpu-prometheus-exporter:1.0.0
```
