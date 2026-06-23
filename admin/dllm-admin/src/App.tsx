import React from 'react'

function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            dllm Admin
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            AI Box 管理後台
          </p>
        </div>
      </header>
      <main className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
        <div className="rounded-lg bg-white p-6 shadow">
          <h2 className="text-lg font-medium text-gray-900">
            歡迎使用 dllm
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            管理後台正在開發中。請使用 API 端點進行操作。
          </p>
          <div className="mt-4 space-y-2">
            <div className="rounded-md bg-blue-50 p-4">
              <p className="text-sm text-blue-800">
                API 端點: <code className="font-mono">http://localhost:11400/v1</code>
              </p>
            </div>
            <div className="rounded-md bg-green-50 p-4">
              <p className="text-sm text-green-800">
                健康檢查: <code className="font-mono">GET /health</code>
              </p>
            </div>
          </div>
        </div>
      </main>
    </div>
  )
}

export default App
