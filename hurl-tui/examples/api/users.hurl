# Get all users
GET https://jsonplaceholder.typicode.com/users
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$" count == 10
jsonpath "$[0].id" == 1
jsonpath "$[0].name" exists
jsonpath "$[0].email" contains "@"
