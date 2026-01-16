# Get all todos
GET https://jsonplaceholder.typicode.com/todos
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$" count == 200


# Get todos for user 1
GET https://jsonplaceholder.typicode.com/users/1/todos
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$" count == 20
jsonpath "$[0].userId" == 1
jsonpath "$[0].completed" isBoolean


# Get single todo
GET https://jsonplaceholder.typicode.com/todos/1
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$.id" == 1
jsonpath "$.title" exists
jsonpath "$.completed" == false
