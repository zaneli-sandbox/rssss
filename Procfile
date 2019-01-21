front: sh -c "cd frontend && elm-app start"
back: sh -c "cd backend && systemfd --no-pid -s http::${RSSSS_BACKEND_PORT} -- cargo watch -x run"
