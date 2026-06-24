#!/usr/bin/env python3
"""
dllm 效能測試套件
測試項目：算力（TPS）、速度（TTFT+延遲）、穩定性（壓力測試）

使用方式：
  python3 benchmark.py                          # 完整測試
  python3 benchmark.py --mode tps               # 只測算力
  python3 benchmark.py --mode latency           # 只測延遲
  python3 benchmark.py --mode stability         # 只測穩定性
"""

import argparse
import json
import time
import sys
import statistics
from datetime import datetime
from typing import List, Dict
import urllib.request
import urllib.error

API_URL = "http://localhost:11400/v1/chat/completions"
MODEL = "/home/daniel/.dllm/models/Qwen2.5-0.5B-Instruct"
WARMUP = 3
ROUNDS = 10

def call_llm(prompt: str, max_tokens: int = 100, stream: bool = False) -> Dict:
    data = json.dumps({
        "model": MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": 0.7,
        "stream": stream,
    }).encode()
    req = urllib.request.Request(API_URL, data=data,
        headers={"Content-Type": "application/json"},
        method="POST")
    try:
        resp = urllib.request.urlopen(req, timeout=300)
        return json.loads(resp.read())
    except Exception as e:
        return {"error": str(e)}

def test_tps():
    """測試 Token Per Second（算力）"""
    print("\n" + "="*60)
    print("📊 算力測試（Tokens Per Second）")
    print("="*60)

    tokens_output = 200
    results = []
    
    for i in range(WARMUP + ROUNDS):
        start = time.time()
        resp = call_llm("請寫一篇關於人工智慧的短文，約200字。", max_tokens=tokens_output)
        elapsed = time.time() - start
        
        if "error" in resp:
            print(f"  ❌ [{i+1}] 錯誤: {resp['error']}")
            continue
        
        usage = resp.get("usage", {})
        completion_tokens = usage.get("completion_tokens", 0)
        tps = completion_tokens / elapsed if elapsed > 0 else 0
        
        if i >= WARMUP:
            results.append(tps)
            print(f"  [{i+1-WARMUP}/{ROUNDS}] {completion_tokens} tokens / {elapsed:.2f}s = {tps:.2f} TPS")
    
    if results:
        avg = statistics.mean(results)
        print(f"\n  ✅ 平均生成速度: {avg:.2f} TPS")
        print(f"  最高: {max(results):.2f} TPS")
        print(f"  最低: {min(results):.2f} TPS")
    return results

def test_latency():
    """測試延遲（TTFT + 各階段耗時）"""
    print("\n" + "="*60)
    print("⏱️  延遲測試（TTFT + 響應時間）")
    print("="*60)

    prompts = ["你好", "請簡單介紹一下你自己。", "1+1=?", "今天天氣如何？"]
    results = []

    for prompt in prompts:
        for _ in range(3):  # 每個 prompt 測 3 次
            start = time.time()
            resp = call_llm(prompt, max_tokens=50)
            elapsed = time.time() - start
            
            if "error" in resp:
                continue
            
            usage = resp.get("usage", {})
            prompt_tokens = usage.get("prompt_tokens", 0)
            completion_tokens = usage.get("completion_tokens", 0)
            content = resp.get("choices", [{}])[0].get("message", {}).get("content", "")[:50]
            
            results.append(elapsed)
            print(f"  [{prompt[:20]:20s}] {elapsed:.2f}s | prompt={prompt_tokens} | output={completion_tokens} | {content}")
    
    if results:
        avg = statistics.mean(results)
        p99 = sorted(results)[int(len(results)*0.99)]
        p95 = sorted(results)[int(len(results)*0.95)]
        print(f"\n  ✅ 平均延遲: {avg:.2f}s")
        print(f"  P95: {p95:.2f}s")
        print(f"  P99: {p99:.2f}s")
    return results

def test_stability():
    """測試穩定性（連續壓力測試）"""
    print("\n" + "="*60)
    print("🔧 穩定性測試（連續請求 + 錯誤率）")
    print("="*60)

    concurrent = 5
    total_requests = concurrent * 4
    results = []
    errors = 0

    prompts = [
        "Hello!", "你好嗎？", "What is AI?", "台灣的天氣如何？",
        "寫一首詩", "解釋量子力學", "1+2+3+4+5=?", "今天星期幾？",
        "什麼是API？", "Python 如何讀取檔案？", "Tell me a joke",
        "蘋果是什麼顏色？", "2*3*4=?", "說一個笑話", "How are you?",
        "明天會更好嗎？", "1兆是多少？", "介紹台灣", "Hello World",
        "翻譯成英文：你好",
    ]

    start_time = time.time()
    for i in range(total_requests):
        prompt = prompts[i % len(prompts)]
        try:
            t0 = time.time()
            resp = call_llm(prompt, max_tokens=100)
            latency = time.time() - t0
            if "error" in resp:
                errors += 1
                print(f"  ❌ [{i+1}] 錯誤: {resp['error']}")
            else:
                results.append(latency)
                print(f"  ✅ [{i+1}/{total_requests}] {latency:.2f}s")
        except Exception as e:
            errors += 1
            print(f"  ❌ [{i+1}] 例外: {e}")

    total_time = time.time() - start_time

    print(f"\n  總請求: {total_requests}")
    print(f"  成功: {len(results)}")
    print(f"  失敗: {errors}")
    print(f"  成功率: {(len(results)/total_requests)*100:.1f}%")
    print(f"  總耗時: {total_time:.1f}s")
    print(f"  平均回應: {total_time/total_requests:.2f}s/req")

    if results:
        avg = statistics.mean(results)
        print(f"  平均延遲: {avg:.2f}s")
    
    return {"total": total_requests, "success": len(results), "errors": errors, "time": total_time}

def main():
    global API_URL, MODEL
    parser = argparse.ArgumentParser(description="dllm 效能測試")
    parser.add_argument("--mode", choices=["tps", "latency", "stability", "all"],
                       default="all", help="測試模式")
    parser.add_argument("--api", default=API_URL, help="API 端點")
    args = parser.parse_args()

    API_URL = args.api

    print(f"\n🚀 dllm 效能測試套件")
    print(f"   時間: {datetime.now().isoformat()}")
    print(f"   API: {API_URL}")
    print(f"   模型: {MODEL}")

    # 先確認服務是否活著
    try:
        urllib.request.urlopen("http://localhost:11400/health", timeout=5)
    except:
        print("\n❌ 無法連接 dllm-core (port 11400)，請先啟動服務")
        sys.exit(1)

    if args.mode in ("tps", "all"):
        test_tps()
    if args.mode in ("latency", "all"):
        test_latency()
    if args.mode in ("stability", "all"):
        test_stability()

    print("\n" + "="*60)
    print("✅ 測試完成")
    print("="*60)

if __name__ == "__main__":
    main()
