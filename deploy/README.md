# Pratrol Blue/Green on One Server

This setup runs two `systemd` instances (`blue` and `green`) on different ports and uses nginx to switch traffic.

## Files in this folder

- `deploy-bluegreen.sh`: build + install + symlink swap + restart inactive color (+ optional traffic switch).
- `systemd/pratrol@.service`: templated unit for `pratrol@blue` and `pratrol@green`.
- `systemd/common.env.example`: shared env values.
- `systemd/blue.env.example`: blue instance port.
- `systemd/green.env.example`: green instance port.
- `nginx/snippets/pratrol-blue.conf`: blue upstream target.
- `nginx/snippets/pratrol-green.conf`: green upstream target.
- `nginx/snippets/pratrol-active.conf`: include target for active color (usually a symlink).
- `nginx-api.pratrol.com-bluegreen.conf`: nginx site config using the active snippet.

## One-time setup

1. Copy unit and env files:

```bash
sudo cp deploy/systemd/pratrol@.service /etc/systemd/system/
sudo mkdir -p /etc/pratrol
sudo cp deploy/systemd/common.env.example /etc/pratrol/common.env
sudo cp deploy/systemd/blue.env.example /etc/pratrol/blue.env
sudo cp deploy/systemd/green.env.example /etc/pratrol/green.env
```

2. Copy nginx snippets and site config:

```bash
sudo mkdir -p /etc/nginx/snippets
sudo cp deploy/nginx/snippets/pratrol-blue.conf /etc/nginx/snippets/
sudo cp deploy/nginx/snippets/pratrol-green.conf /etc/nginx/snippets/
sudo ln -sfn /etc/nginx/snippets/pratrol-blue.conf /etc/nginx/snippets/pratrol-active.conf
sudo cp deploy/nginx-api.pratrol.com-bluegreen.conf /etc/nginx/sites-available/api.pratrol.com.conf
sudo ln -sfn /etc/nginx/sites-available/api.pratrol.com.conf /etc/nginx/sites-enabled/api.pratrol.com.conf
sudo nginx -t
sudo systemctl reload nginx
```

3. Prepare release roots and reload systemd:

```bash
sudo install -d -m 0755 -o pratrol -g pratrol /opt/pratrol/releases
sudo systemctl daemon-reload
```

4. Bootstrap both colors (creates `current-blue` and `current-green` symlinks):

```bash
bash deploy/deploy-bluegreen.sh --color blue
bash deploy/deploy-bluegreen.sh --color green --skip-build
sudo systemctl enable --now pratrol@blue.service pratrol@green.service
```

## Deploy to inactive color and switch traffic

```bash
bash deploy/deploy-bluegreen.sh --color auto --switch
```

`--color auto` deploys to the opposite of currently active nginx color.

## Deploy to a specific color without switch

```bash
bash deploy/deploy-bluegreen.sh --color green
```

## Rollback (traffic only)

```bash
sudo ln -sfn /etc/nginx/snippets/pratrol-blue.conf /etc/nginx/snippets/pratrol-active.conf
sudo nginx -t
sudo systemctl reload nginx
```

## Notes

- `deploy-bluegreen.sh` uses `install` + symlink swap, not `cp` over a live binary.
- The script restarts only one color service (`pratrol@blue` or `pratrol@green`).
- It tries `http://LISTEN_ADDR/health` if `LISTEN_ADDR` exists in `/etc/pratrol/<color>.env`.
