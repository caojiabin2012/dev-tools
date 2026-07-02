use std::fs;
use std::path::Path;

use crate::stack::store::resolve_www_root;
use crate::stack::types::StackStore;

const TEST_PHP: &str = r#"<?php
header('Content-Type: text/html; charset=utf-8');
$config = require __DIR__ . '/devtools-env.php';

$mysqlOk = false;
$mysqlRows = [];
$redisOk = false;
$redisInfo = [];
$errors = [];

// --- MySQL（PDO）---
try {
    if (!extension_loaded('pdo_mysql')) {
        throw new RuntimeException('pdo_mysql 扩展未启用');
    }
    $m = $config['mysql'];
    $dsn = sprintf(
        'mysql:host=%s;port=%d;dbname=%s;charset=utf8mb4',
        $m['host'],
        $m['port'],
        $m['database']
    );
    $pdo = new PDO($dsn, $m['user'], $m['password'], [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
        PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
    ]);
    $version = $pdo->query('SELECT VERSION() AS v')->fetch();
    $mysqlRows = $pdo->query('SELECT id, name, email, created_at FROM demo_users ORDER BY id')->fetchAll();
    $mysqlOk = true;
    $mysqlInfo = [
        'driver' => 'PDO (pdo_mysql)',
        'dsn' => $dsn,
        'user' => $m['user'],
        'version' => $version['v'] ?? '',
        'rows' => count($mysqlRows),
    ];
} catch (Throwable $e) {
    $errors[] = 'MySQL: ' . $e->getMessage();
    $mysqlInfo = ['connected' => false];
}

// --- Redis ---
try {
    if (!extension_loaded('redis')) {
        throw new RuntimeException('redis 扩展未启用');
    }
    $r = $config['redis'];
    $redis = new Redis();
    if (!$redis->connect($r['host'], (int) $r['port'], 2.0)) {
        throw new RuntimeException('Redis 连接失败');
    }
    $redis->set('devtools:test', 'hello from dev-tools');
    $redisInfo = [
        'driver' => 'phpredis',
        'host' => $r['host'],
        'port' => $r['port'],
        'ping' => $redis->ping(),
        'devtools:test' => $redis->get('devtools:test'),
    ];
    $redisOk = true;
} catch (Throwable $e) {
    $errors[] = 'Redis: ' . $e->getMessage();
}

?><!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <title>dev-tools 环境联调测试</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 880px; margin: 32px auto; padding: 0 16px; color: #18181b; }
    h1 { font-size: 1.35rem; }
    .summary { display: flex; gap: 12px; flex-wrap: wrap; margin: 16px 0; }
    .pill { padding: 8px 14px; border-radius: 999px; font-size: 13px; border: 1px solid #e4e4e7; }
    .pill.ok { background: #ecfdf5; color: #059669; border-color: #a7f3d0; }
    .pill.fail { background: #fef2f2; color: #dc2626; border-color: #fecaca; }
    section { margin: 24px 0; padding: 16px; border: 1px solid #e4e4e7; border-radius: 12px; background: #fafafa; }
    table { width: 100%; border-collapse: collapse; font-size: 14px; }
    th, td { border: 1px solid #e4e4e7; padding: 8px 10px; text-align: left; }
    th { background: #f4f4f5; }
    .err { color: #dc2626; background: #fef2f2; padding: 12px; border-radius: 8px; margin: 8px 0; }
    code { background: #f4f4f5; padding: 2px 6px; border-radius: 4px; }
    pre { background: #18181b; color: #fafafa; padding: 12px; border-radius: 8px; overflow: auto; font-size: 13px; }
  </style>
</head>
<body>
  <h1>dev-tools 环境联调测试</h1>
  <p>MySQL 使用 <code>PDO</code> 连接；Redis 使用 <code>phpredis</code> 扩展。</p>

  <div class="summary">
    <span class="pill <?= $mysqlOk ? 'ok' : 'fail' ?>">MySQL <?= $mysqlOk ? '已连接' : '未连接' ?></span>
    <span class="pill <?= $redisOk ? 'ok' : 'fail' ?>">Redis <?= $redisOk ? '已连接' : '未连接' ?></span>
  </div>

  <?php if ($errors): ?>
    <?php foreach ($errors as $err): ?>
      <div class="err"><?= htmlspecialchars($err) ?></div>
    <?php endforeach; ?>
  <?php endif; ?>

  <section>
    <h2>MySQL · test.demo_users</h2>
    <?php if ($mysqlOk): ?>
      <pre><?= htmlspecialchars(json_encode($mysqlInfo, JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE)) ?></pre>
      <table>
        <thead><tr><th>ID</th><th>姓名</th><th>邮箱</th><th>创建时间</th></tr></thead>
        <tbody>
        <?php foreach ($mysqlRows as $row): ?>
          <tr>
            <td><?= (int)$row['id'] ?></td>
            <td><?= htmlspecialchars($row['name']) ?></td>
            <td><?= htmlspecialchars($row['email']) ?></td>
            <td><?= htmlspecialchars($row['created_at']) ?></td>
          </tr>
        <?php endforeach; ?>
        </tbody>
      </table>
    <?php else: ?>
      <p>请确认 MariaDB 已启动，且 test 库 / demo_users 表已初始化（启动 MySQL 时会自动创建）。</p>
    <?php endif; ?>
  </section>

  <section>
    <h2>Redis</h2>
    <?php if ($redisOk): ?>
      <pre><?= htmlspecialchars(json_encode($redisInfo, JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE)) ?></pre>
    <?php else: ?>
      <p>请确认 Redis 已启动（默认 127.0.0.1:6379）。</p>
    <?php endif; ?>
  </section>

  <p><a href="index.php">← 返回首页</a></p>
</body>
</html>
"#;

/// 写入网站根目录下的 devtools-env.php、test.php，并确保 index 页面存在。
pub fn sync_site_files(store: &StackStore, install_root: &Path) -> Result<(), String> {
    let www = resolve_www_root(install_root, &store.settings);
    fs::create_dir_all(&www).map_err(|e| e.to_string())?;

    let mysql_port = store.mysql.as_ref().map(|m| m.port).unwrap_or(3307);
    let redis_port = store.redis.as_ref().map(|r| r.port).unwrap_or(6379);

    let env_php = format!(
        r#"<?php
return [
    'mysql' => [
        'host' => '127.0.0.1',
        'port' => {mysql_port},
        'user' => 'root',
        'password' => '',
        'database' => 'test',
    ],
    'redis' => [
        'host' => '127.0.0.1',
        'port' => {redis_port},
    ],
];
"#
    );

    fs::write(www.join("devtools-env.php"), env_php).map_err(|e| e.to_string())?;
    fs::write(www.join("test.php"), TEST_PHP).map_err(|e| e.to_string())?;
    ensure_index_files(&www)?;
    Ok(())
}

fn ensure_index_files(www: &Path) -> Result<(), String> {
    fs::write(
        &www.join("index.php"),
        r#"<?php
header('Content-Type: text/html; charset=utf-8');
echo '<!DOCTYPE html><html><head><meta charset="utf-8"><title>dev-tools 本地环境</title>';
echo '<style>body{font-family:system-ui,sans-serif;max-width:720px;margin:48px auto;padding:0 16px;color:#222}';
echo 'code{background:#f4f4f5;padding:2px 6px;border-radius:4px}a{color:#2563eb}</style></head><body>';
echo '<h1>dev-tools 本地环境</h1>';
echo '<p>Nginx + PHP 运行正常。网站根目录：<code>' . htmlspecialchars(__DIR__) . '</code></p>';
echo '<p><a href="test.php"><strong>MySQL + Redis 联调测试</strong></a></p>';
echo '<p><a href="index.php?phpinfo=1">查看 PHP 信息</a></p>';
if (isset($_GET['phpinfo'])) { echo '<hr><h2>PHP 信息</h2>'; phpinfo(); }
"#,
    )
    .map_err(|e| e.to_string())?;

    fs::write(
        &www.join("index.html"),
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="utf-8"><title>dev-tools 本地环境</title></head>
<body>
<h1>dev-tools 本地环境</h1>
<p><a href="test.php">MySQL + Redis 联调测试</a> · <a href="index.php">PHP 首页</a></p>
</body>
</html>
"#,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
