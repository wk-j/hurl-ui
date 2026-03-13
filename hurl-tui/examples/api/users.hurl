# Get all users
GET {{base_url}}/users
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$" count == 10
jsonpath "$[0].id" == {{user_id}}
jsonpath "$[0].name" exists
jsonpath "$[0].email" contains "@"
