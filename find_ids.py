import requests
import re
import sys

url = "https://uspdigital.usp.br/rucard/dwr/call/plaincall/CardapioControleDWR.obterCardapioRestUSP.dwr"
headers = {"Content-Type": "text/plain"}

valid_ids = []

for i in range(100):
    body = f"callCount=1\nwindowName=\nc0-scriptName=CardapioControleDWR\nc0-methodName=obterCardapioRestUSP\nc0-id=0\nc0-param0=string:{i}\nbatchId=0\ninstanceId=0\npage=%2Frucard%2FJsp%2FcardapioSAS.jsp%3Fcodrtn%3D{i}\nscriptSessionId="
    try:
        response = requests.post(url, headers=headers, data=body, timeout=5)
        # Se contiver cdpdia: ou tiprfi: (campos do cardápio) é provável que seja válido
        if "cdpdia:" in response.text:
            # Tentar extrair o nome do restaurante se possível
            # O nome do restaurante geralmente não está nesse DWR, mas sim em outro.
            # No entanto, se o cardápio existe, o ID é válido.
            valid_ids.append(i)
            print(f"ID {i} is valid")
    except Exception as e:
        print(f"Error checking ID {i}: {e}")

print("\nValid IDs found:", valid_ids)
