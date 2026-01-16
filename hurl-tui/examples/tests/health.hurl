# Health check - verify API is responding
GET https://jsonplaceholder.typicode.com/posts/1

HTTP 200
[Asserts]
status == 200
header "Content-Type" contains "application/json"
duration < 5000
