# Build stage
FROM golang:1.21-alpine AS builder

# 安装 CGO 依赖 (SQLite 需要)
RUN apk add --no-cache gcc musl-dev

WORKDIR /app

# 复制依赖文件
COPY go.mod go.sum ./
RUN go mod download

# 复制源代码
COPY . .

# 构建
RUN CGO_ENABLED=1 GOOS=linux go build -o zeni-x cmd/server/main.go

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# 复制构建产物
COPY --from=builder /app/zeni-x .
COPY --from=builder /app/configs ./configs

# 创建数据目录
RUN mkdir -p /data

EXPOSE 8080

CMD ["./zeni-x", "-config", "configs/test.yaml"]

