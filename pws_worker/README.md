# Phi-WebHook-Server-Worker

## 环境变量
| 环境变量名       | 类型            | 是否必需 | 说明                     | 示例值                                |
|------------------|-----------------|----------|--------------------------|---------------------------------------|
| `FILE_URL_TEMPLATE` | `String`        | 是       | 文件 URL 模板            | `https://localhost/1.1/files/{file_obj_id}` |
| `SIGN_KEY`       | `Secret / String` | 是       | 签名密钥                 | `your-secret`                         |
| `LOG_LEVEL`      | `String`        | 是       | 日志等级                 | `DEBUG`                               |