<details>
  <summary>API server checks: history with httpie(http cli)e</summary>

- http GET :8080/login
- http GET :8080/login email=abc@gmail.com password=1234
- http GET :8080/login email=abc@gmail.com password=1234
- http GET :8080/users
- http GET :8080/login email=nk@abc.com password=secret
- http GET :8080/login email=nk@abc.com password=secret
- http GET :8080/whoami
- http GET :8080/whoami 'Cookie:auth_token=f24ac565-9956-4918-808e-b152358317ca'
- http GET :8080/login email=nk@abc.com password=secret
- http GET :8080/login email=nk@abc.com password=secret
- http GET :8080/users
- http GET :8080/login email=xyz@abc.com password=secret
- http GET :8080/whoami 'Cookie:auth_token=c0ab5aef-d1ea-499e-b826-d80db7fa1ebd'
- http GET :8080/whoami 'Cookie:auth_token=c0ab5aef-d1ea-499e-b826-d80db7fa1ebd'
- http GET :8080/login email=xyz@abc.com password=secret
- http GET :8080/whoami 'Cookie:auth_token=c0850e09-6a22-429e-a34d-16e69dc8930b'
- http GET :8080/whoami 'Cookie:auth_token=c0850e09-6a22-429e-a34d-36e69dc8930b'
- http GET :8080/users
- http GET :8080/login email=nachiket@abc.com password=secret
- http GET :8080/login email=nachiket@abc.com password=asfdasdfasdf
- http GET :8080/login email=nachiket@abc.com password=secret
- http GET :8080/whoami 'Cookie:auth_token=f2b4082d-6ca9-4651-9387-22e7d70872b6'
- http POST :8080/signup email=nk@abc.com password=secret
- http POST :8080/signup email=hello@abc.com password=secret
- http GET :8080/login email=hello@abc.com password=secret
- http GET :8080/whoami 'Cookie:auth_token=8762cbd1-044f-42b9-860a-afa7fb75a3ea'

</details>
