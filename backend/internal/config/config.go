package config

import (
	"os"
	"strings"

	"github.com/spf13/viper"
)

// Config 应用配置
type Config struct {
	Server  ServerConfig  `mapstructure:"server"`
	SQLite  SQLiteConfig  `mapstructure:"sqlite"`
	MySQL   MySQLConfig   `mapstructure:"mysql"`
	Redis   RedisConfig   `mapstructure:"redis"`
	MongoDB MongoDBConfig `mapstructure:"mongodb"`
	MinIO   MinIOConfig   `mapstructure:"minio"`
}

// ServerConfig 服务器配置
type ServerConfig struct {
	Port string `mapstructure:"port"`
	Mode string `mapstructure:"mode"` // debug, release
}

// SQLiteConfig SQLite 配置
type SQLiteConfig struct {
	Path string `mapstructure:"path"`
}

// MySQLConfig MySQL 配置
type MySQLConfig struct {
	Host     string `mapstructure:"host"`
	Port     int    `mapstructure:"port"`
	User     string `mapstructure:"user"`
	Password string `mapstructure:"password"`
	Database string `mapstructure:"database"`
}

// RedisConfig Redis 配置
type RedisConfig struct {
	Host     string `mapstructure:"host"`
	Port     int    `mapstructure:"port"`
	Password string `mapstructure:"password"`
	DB       int    `mapstructure:"db"`
}

// MongoDBConfig MongoDB 配置
type MongoDBConfig struct {
	Host     string `mapstructure:"host"`
	Port     int    `mapstructure:"port"`
	User     string `mapstructure:"user"`
	Password string `mapstructure:"password"`
	Database string `mapstructure:"database"`
}

// MinIOConfig MinIO 配置
type MinIOConfig struct {
	Endpoint  string `mapstructure:"endpoint"`
	AccessKey string `mapstructure:"access_key"`
	SecretKey string `mapstructure:"secret_key"`
	UseSSL    bool   `mapstructure:"use_ssl"`
}

// Load 加载配置文件
func Load(configPath string) (*Config, error) {
	v := viper.New()

	// 设置配置文件
	v.SetConfigFile(configPath)
	v.SetConfigType("yaml")

	// 支持环境变量
	v.AutomaticEnv()
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))

	// 读取配置文件
	if err := v.ReadInConfig(); err != nil {
		return nil, err
	}

	var cfg Config
	if err := v.Unmarshal(&cfg); err != nil {
		return nil, err
	}

	// 环境变量覆盖
	cfg.MySQL.Password = getEnvOrDefault("MYSQL_ROOT_PASSWORD", cfg.MySQL.Password)
	cfg.Redis.Password = getEnvOrDefault("REDIS_PASSWORD", cfg.Redis.Password)
	cfg.MongoDB.Password = getEnvOrDefault("MONGODB_PASSWORD", cfg.MongoDB.Password)
	cfg.MinIO.SecretKey = getEnvOrDefault("MINIO_SECRET_KEY", cfg.MinIO.SecretKey)

	return &cfg, nil
}

func getEnvOrDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

