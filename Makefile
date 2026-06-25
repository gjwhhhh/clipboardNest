# 剪切板管理器 - 常用命令

.PHONY: dev start stop build test clean help

# 默认目标
help:
	@echo "可用命令:"
	@echo "  make dev        - 启动开发模式（带热重载）"
	@echo "  make start      - 同 make dev"
	@echo "  make stop       - 停止运行中的应用"
	@echo "  make build      - 构建生产版本"
	@echo "  make test       - 运行所有测试"
	@echo "  make clean      - 清理构建文件"
	@echo "  make restart    - 重启应用"
	@echo "  make logs       - 查看应用日志"

# 启动开发模式
dev start:
	npm run tauri dev

# 停止应用
stop:
	@pkill -f clipboard-manager 2>/dev/null || true
	@echo "应用已停止"

# 重启应用
restart: stop
	@sleep 1
	@make dev

# 构建生产版本
build:
	npm run tauri build

# 运行测试
test:
	cd src-tauri && cargo test

# 清理构建文件
clean:
	cd src-tauri && cargo clean
	rm -rf dist node_modules/.cache

# 查看日志
logs:
	@echo "应用日志:"
	@echo "RUST_LOG=info npm run tauri dev"
