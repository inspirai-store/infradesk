package config

import (
	"os"
	"strconv"
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
// 环境变量优先级高于配置文件
// 如果 configPath 为空，则只使用默认值和环境变量
func Load(configPath string) (*Config, error) {
	v := viper.New()

	// 设置默认值
	setDefaults(v)

	// 如果提供了配置文件路径，读取配置文件
	if configPath != "" {
		v.SetConfigFile(configPath)
		v.SetConfigType("yaml")
		// 忽略配置文件不存在的错误，因为环境变量可能已经提供了所有配置
		_ = v.ReadInConfig()
	}

	// 支持环境变量（环境变量会覆盖配置文件和默认值）
	v.AutomaticEnv()
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))

	var cfg Config
	if err := v.Unmarshal(&cfg); err != nil {
		return nil, err
	}

	// 显式用环境变量覆盖（确保优先级）
	// Server 配置
	cfg.Server.Port = getEnvOrDefault("SERVER_PORT", cfg.Server.Port)
	cfg.Server.Mode = getEnvOrDefault("SERVER_MODE", cfg.Server.Mode)

	// SQLite 配置
	cfg.SQLite.Path = getEnvOrDefault("SQLITE_PATH", cfg.SQLite.Path)

	// MySQL 配置
	cfg.MySQL.Host = getEnvOrDefault("MYSQL_HOST", cfg.MySQL.Host)
	cfg.MySQL.Port = getEnvIntOrDefault("MYSQL_PORT", cfg.MySQL.Port)
	cfg.MySQL.User = getEnvOrDefault("MYSQL_USER", cfg.MySQL.User)
	cfg.MySQL.Password = getEnvOrDefault("MYSQL_PASSWORD", cfg.MySQL.Password)
	cfg.MySQL.Database = getEnvOrDefault("MYSQL_DATABASE", cfg.MySQL.Database)

	// Redis 配置
	cfg.Redis.Host = getEnvOrDefault("REDIS_HOST", cfg.Redis.Host)
	cfg.Redis.Port = getEnvIntOrDefault("REDIS_PORT", cfg.Redis.Port)
	cfg.Redis.Password = getEnvOrDefault("REDIS_PASSWORD", cfg.Redis.Password)
	cfg.Redis.DB = getEnvIntOrDefault("REDIS_DB", cfg.Redis.DB)

	// MongoDB 配置
	cfg.MongoDB.Host = getEnvOrDefault("MONGODB_HOST", cfg.MongoDB.Host)
	cfg.MongoDB.Port = getEnvIntOrDefault("MONGODB_PORT", cfg.MongoDB.Port)
	cfg.MongoDB.User = getEnvOrDefault("MONGODB_USER", cfg.MongoDB.User)
	cfg.MongoDB.Password = getEnvOrDefault("MONGODB_PASSWORD", cfg.MongoDB.Password)
	cfg.MongoDB.Database = getEnvOrDefault("MONGODB_DATABASE", cfg.MongoDB.Database)

	// MinIO 配置
	cfg.MinIO.Endpoint = getEnvOrDefault("MINIO_ENDPOINT", cfg.MinIO.Endpoint)
	cfg.MinIO.AccessKey = getEnvOrDefault("MINIO_ACCESS_KEY", cfg.MinIO.AccessKey)
	cfg.MinIO.SecretKey = getEnvOrDefault("MINIO_SECRET_KEY", cfg.MinIO.SecretKey)
	cfg.MinIO.UseSSL = getEnvBoolOrDefault("MINIO_USE_SSL", cfg.MinIO.UseSSL)

	return &cfg, nil
}

// setDefaults 设置默认配置值
func setDefaults(v *viper.Viper) {
	// Server 默认值
	v.SetDefault("server.port", "8080")
	v.SetDefault("server.mode", "release")

	// SQLite 默认值
	v.SetDefault("sqlite.path", "./data/zeni-x.db")

	// MySQL 默认值
	v.SetDefault("mysql.host", "")
	v.SetDefault("mysql.port", 3306)
	v.SetDefault("mysql.user", "root")
	v.SetDefault("mysql.password", "")
	v.SetDefault("mysql.database", "")

	// Redis 默认值
	v.SetDefault("redis.host", "")
	v.SetDefault("redis.port", 6379)
	v.SetDefault("redis.password", "")
	v.SetDefault("redis.db", 0)

	// MongoDB 默认值
	v.SetDefault("mongodb.host", "")
	v.SetDefault("mongodb.port", 27017)
	v.SetDefault("mongodb.user", "")
	v.SetDefault("mongodb.password", "")
	v.SetDefault("mongodb.database", "")

	// MinIO 默认值
	v.SetDefault("minio.endpoint", "")
	v.SetDefault("minio.access_key", "")
	v.SetDefault("minio.secret_key", "")
	v.SetDefault("minio.use_ssl", false)
}

func getEnvOrDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func getEnvIntOrDefault(key string, defaultValue int) int {
	if value := os.Getenv(key); value != "" {
		if intVal, err := strconv.Atoi(value); err == nil {
			return intVal
		}
	}
	return defaultValue
}

func getEnvBoolOrDefault(key string, defaultValue bool) bool {
	if value := os.Getenv(key); value != "" {
		if boolVal, err := strconv.ParseBool(value); err == nil {
			return boolVal
		}
	}
	return defaultValue
}

