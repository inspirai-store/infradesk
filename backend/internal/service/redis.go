package service

import (
	"context"
	"encoding/json"
	"fmt"
	"strconv"
	"time"

	"github.com/go-redis/redis/v8"
	"github.com/zeni-x/backend/internal/store"
)

// RedisService Redis 服务
type RedisService struct {
	// No longer holds global config - connections are passed dynamically
}

// NewRedisService 创建 Redis 服务
func NewRedisService() *RedisService {
	return &RedisService{}
}

// connect 创建 Redis 连接
func (s *RedisService) connect(conn *store.Connection) (*redis.Client, error) {
	// Parse DB index from DatabaseName field (stored as string)
	dbIndex := 0
	if conn.DatabaseName != "" {
		if parsed, err := strconv.Atoi(conn.DatabaseName); err == nil {
			dbIndex = parsed
		}
	}

	client := redis.NewClient(&redis.Options{
		Addr:     fmt.Sprintf("%s:%d", conn.Host, conn.Port),
		Password: conn.Password,
		DB:       dbIndex,
	})

	ctx := context.Background()
	if err := client.Ping(ctx).Err(); err != nil {
		return nil, err
	}

	return client, nil
}

// RedisInfo Redis 服务器信息
type RedisInfo struct {
	Version          string `json:"version"`
	Host             string `json:"host"`
	Port             int    `json:"port"`
	Connected        bool   `json:"connected"`
	UsedMemory       string `json:"used_memory"`
	TotalKeys        int64  `json:"total_keys"`
	ConnectedClients int64  `json:"connected_clients"`
}

// GetInfo 获取 Redis 信息
func (s *RedisService) GetInfo(conn *store.Connection) (*RedisInfo, error) {
	client, err := s.connect(conn)
	if err != nil {
		return &RedisInfo{
			Host:      conn.Host,
			Port:      conn.Port,
			Connected: false,
		}, nil
	}
	defer client.Close()

	ctx := context.Background()

	// 获取 INFO
	info, err := client.Info(ctx).Result()
	if err != nil {
		return nil, err
	}

	// 解析版本信息
	var version, usedMemory string
	var connectedClients int64
	for _, line := range splitLines(info) {
		if len(line) > 14 && line[:14] == "redis_version:" {
			version = line[14:]
		}
		if len(line) > 17 && line[:17] == "used_memory_human:" {
			usedMemory = line[17:]
		}
		if len(line) > 18 && line[:18] == "connected_clients:" {
			fmt.Sscanf(line[18:], "%d", &connectedClients)
		}
	}

	// 获取 Key 数量
	dbSize, _ := client.DBSize(ctx).Result()

	return &RedisInfo{
		Version:          version,
		Host:             conn.Host,
		Port:             conn.Port,
		Connected:        true,
		UsedMemory:       usedMemory,
		TotalKeys:        dbSize,
		ConnectedClients: connectedClients,
	}, nil
}

// splitLines 分割行
func splitLines(s string) []string {
	var lines []string
	start := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '\n' {
			line := s[start:i]
			if len(line) > 0 && line[len(line)-1] == '\r' {
				line = line[:len(line)-1]
			}
			lines = append(lines, line)
			start = i + 1
		}
	}
	if start < len(s) {
		lines = append(lines, s[start:])
	}
	return lines
}

// KeyInfo Key 信息
type KeyInfo struct {
	Key   string      `json:"key"`
	Type  string      `json:"type"`
	TTL   int64       `json:"ttl"`
	Value interface{} `json:"value,omitempty"`
}

// KeysResult Keys 列表结果
type KeysResult struct {
	Keys   []KeyInfo `json:"keys"`
	Cursor uint64    `json:"cursor"`
	Total  int64     `json:"total"`
}

// ListKeys 列出 Keys
func (s *RedisService) ListKeys(conn *store.Connection, pattern string, cursor uint64, count int64) (*KeysResult, error) {
	client, err := s.connect(conn)
	if err != nil {
		return nil, err
	}
	defer client.Close()

	ctx := context.Background()

	if pattern == "" {
		pattern = "*"
	}

	// 使用 SCAN 迭代
	keys, nextCursor, err := client.Scan(ctx, cursor, pattern, count).Result()
	if err != nil {
		return nil, err
	}

	// 获取每个 Key 的类型和 TTL
	var keyInfos []KeyInfo
	for _, key := range keys {
		keyType, _ := client.Type(ctx, key).Result()
		ttl, _ := client.TTL(ctx, key).Result()

		keyInfos = append(keyInfos, KeyInfo{
			Key:  key,
			Type: keyType,
			TTL:  int64(ttl.Seconds()),
		})
	}

	// 获取总数
	total, _ := client.DBSize(ctx).Result()

	return &KeysResult{
		Keys:   keyInfos,
		Cursor: nextCursor,
		Total:  total,
	}, nil
}

// GetKey 获取 Key 详情
func (s *RedisService) GetKey(conn *store.Connection, key string) (*KeyInfo, error) {
	client, err := s.connect(conn)
	if err != nil {
		return nil, err
	}
	defer client.Close()

	ctx := context.Background()

	// 获取类型
	keyType, err := client.Type(ctx, key).Result()
	if err != nil {
		return nil, err
	}

	if keyType == "none" {
		return nil, fmt.Errorf("key not found: %s", key)
	}

	// 获取 TTL
	ttl, _ := client.TTL(ctx, key).Result()

	info := &KeyInfo{
		Key:  key,
		Type: keyType,
		TTL:  int64(ttl.Seconds()),
	}

	// 根据类型获取值
	switch keyType {
	case "string":
		val, err := client.Get(ctx, key).Result()
		if err != nil {
			return nil, err
		}
		info.Value = val

	case "hash":
		val, err := client.HGetAll(ctx, key).Result()
		if err != nil {
			return nil, err
		}
		info.Value = val

	case "list":
		val, err := client.LRange(ctx, key, 0, -1).Result()
		if err != nil {
			return nil, err
		}
		info.Value = val

	case "set":
		val, err := client.SMembers(ctx, key).Result()
		if err != nil {
			return nil, err
		}
		info.Value = val

	case "zset":
		val, err := client.ZRangeWithScores(ctx, key, 0, -1).Result()
		if err != nil {
			return nil, err
		}
		info.Value = val
	}

	return info, nil
}

// SetKeyRequest 设置 Key 请求
type SetKeyRequest struct {
	Key   string      `json:"key"`
	Type  string      `json:"type"`
	Value interface{} `json:"value"`
	TTL   int64       `json:"ttl,omitempty"` // 秒，-1 表示不过期
}

// SetKey 设置 Key
func (s *RedisService) SetKey(conn *store.Connection, req *SetKeyRequest) error {
	client, err := s.connect(conn)
	if err != nil {
		return err
	}
	defer client.Close()

	ctx := context.Background()

	var expiration time.Duration
	if req.TTL > 0 {
		expiration = time.Duration(req.TTL) * time.Second
	}

	switch req.Type {
	case "string":
		val, ok := req.Value.(string)
		if !ok {
			return fmt.Errorf("invalid value type for string")
		}
		return client.Set(ctx, req.Key, val, expiration).Err()

	case "hash":
		val, ok := req.Value.(map[string]interface{})
		if !ok {
			return fmt.Errorf("invalid value type for hash")
		}
		if err := client.Del(ctx, req.Key).Err(); err != nil {
			return err
		}
		for k, v := range val {
			if err := client.HSet(ctx, req.Key, k, v).Err(); err != nil {
				return err
			}
		}
		if expiration > 0 {
			client.Expire(ctx, req.Key, expiration)
		}
		return nil

	case "list":
		val, ok := req.Value.([]interface{})
		if !ok {
			return fmt.Errorf("invalid value type for list")
		}
		if err := client.Del(ctx, req.Key).Err(); err != nil {
			return err
		}
		for _, v := range val {
			if err := client.RPush(ctx, req.Key, v).Err(); err != nil {
				return err
			}
		}
		if expiration > 0 {
			client.Expire(ctx, req.Key, expiration)
		}
		return nil

	case "set":
		val, ok := req.Value.([]interface{})
		if !ok {
			return fmt.Errorf("invalid value type for set")
		}
		if err := client.Del(ctx, req.Key).Err(); err != nil {
			return err
		}
		for _, v := range val {
			if err := client.SAdd(ctx, req.Key, v).Err(); err != nil {
				return err
			}
		}
		if expiration > 0 {
			client.Expire(ctx, req.Key, expiration)
		}
		return nil

	case "zset":
		val, ok := req.Value.([]interface{})
		if !ok {
			return fmt.Errorf("invalid value type for zset")
		}
		if err := client.Del(ctx, req.Key).Err(); err != nil {
			return err
		}
		for _, item := range val {
			m, ok := item.(map[string]interface{})
			if !ok {
				continue
			}
			member, _ := m["member"].(string)
			score, _ := m["score"].(float64)
			if err := client.ZAdd(ctx, req.Key, &redis.Z{Score: score, Member: member}).Err(); err != nil {
				return err
			}
		}
		if expiration > 0 {
			client.Expire(ctx, req.Key, expiration)
		}
		return nil

	default:
		return fmt.Errorf("unsupported type: %s", req.Type)
	}
}

// DeleteKey 删除 Key
func (s *RedisService) DeleteKey(conn *store.Connection, key string) error {
	client, err := s.connect(conn)
	if err != nil {
		return err
	}
	defer client.Close()

	return client.Del(context.Background(), key).Err()
}

// SetTTL 设置 TTL
func (s *RedisService) SetTTL(conn *store.Connection, key string, ttl int64) error {
	client, err := s.connect(conn)
	if err != nil {
		return err
	}
	defer client.Close()

	ctx := context.Background()

	if ttl < 0 {
		// 移除过期时间
		return client.Persist(ctx, key).Err()
	}

	return client.Expire(ctx, key, time.Duration(ttl)*time.Second).Err()
}

// ExportData 导出数据
type ExportData struct {
	Keys []KeyInfo `json:"keys"`
}

// Export 导出数据
func (s *RedisService) Export(conn *store.Connection, keys []string) (*ExportData, error) {
	var keyInfos []KeyInfo

	for _, key := range keys {
		info, err := s.GetKey(conn, key)
		if err != nil {
			continue
		}
		keyInfos = append(keyInfos, *info)
	}

	return &ExportData{Keys: keyInfos}, nil
}

// Import 导入数据
func (s *RedisService) Import(conn *store.Connection, data *ExportData) error {
	for _, keyInfo := range data.Keys {
		req := &SetKeyRequest{
			Key:   keyInfo.Key,
			Type:  keyInfo.Type,
			Value: keyInfo.Value,
			TTL:   keyInfo.TTL,
		}
		if err := s.SetKey(conn, req); err != nil {
			return err
		}
	}
	return nil
}

// ExportJSON 导出为 JSON 字符串
func (s *RedisService) ExportJSON(conn *store.Connection, keys []string) (string, error) {
	data, err := s.Export(conn, keys)
	if err != nil {
		return "", err
	}

	b, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return "", err
	}

	return string(b), nil
}

// ImportJSON 从 JSON 导入
func (s *RedisService) ImportJSON(conn *store.Connection, jsonStr string) error {
	var data ExportData
	if err := json.Unmarshal([]byte(jsonStr), &data); err != nil {
		return err
	}

	return s.Import(conn, &data)
}

// TestConnection 测试连接是否有效
func (s *RedisService) TestConnection(conn *store.Connection) error {
	client, err := s.connect(conn)
	if err != nil {
		return err
	}
	defer client.Close()
	return nil
}
