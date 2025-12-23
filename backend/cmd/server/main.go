package main

import (
	"flag"
	"log"
	"os"

	"github.com/zeni-x/backend/internal/api"
	"github.com/zeni-x/backend/internal/config"
	"github.com/zeni-x/backend/internal/store"
)

func main() {
	// 解析命令行参数
	configPath := flag.String("config", "configs/dev.yaml", "配置文件路径")
	flag.Parse()

	// 加载配置
	cfg, err := config.Load(*configPath)
	if err != nil {
		log.Fatalf("加载配置失败: %v", err)
	}

	// 初始化 SQLite 存储
	db, err := store.NewSQLite(cfg.SQLite.Path)
	if err != nil {
		log.Fatalf("初始化 SQLite 失败: %v", err)
	}
	defer db.Close()

	// 创建并启动路由
	router := api.NewRouter(cfg, db)

	port := cfg.Server.Port
	if port == "" {
		port = "8080"
	}

	log.Printf("🚀 Zeni-X Server starting on port %s", port)
	log.Printf("📊 Mode: %s", cfg.Server.Mode)

	if err := router.Run(":" + port); err != nil {
		log.Fatalf("服务器启动失败: %v", err)
		os.Exit(1)
	}
}

