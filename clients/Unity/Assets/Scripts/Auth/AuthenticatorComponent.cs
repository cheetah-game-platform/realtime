#if UNITY_ANDROID
using Cheetah.Authentication.Android;
#endif
using Cheetah.Authentication.Cookie;
using Cheetah.Platform;
using UnityEngine;
using UnityEngine.UI;

namespace Example.Auth
{
    public class AuthenticatorComponent : MonoBehaviour
    {
        private const string androidWebClientId = "663521173650-gkgrl7aouifjag0j5do14pul1hdqvosm.apps.googleusercontent.com";

        [SerializeField] private Text resultText;
        [SerializeField] private Button androidLoginButton;
        [SerializeField] private Button cookieLoginButton;
        private Connector connector = new Connector("test.dev.cheetah.games", 443, true);

        private void Start()
        {
            androidLoginButton.onClick.AddListener(OnAndroidLogin);
            cookieLoginButton.onClick.AddListener(OnCookieLogin);
        }

        private async void OnCookieLogin()
        {
            var cookieAuthenticator = new CookieAuthenticator(connector);
            //cookieAuthenticator.RemoveLocalCookie();
            var result = await cookieAuthenticator.LoginOrRegister();
            resultText.text = "Login with CookieAuthenticator, register =" + result.RegisteredPlayer + ", player = " + result.Player;
        }

        private async void OnAndroidLogin()
        {
#if UNITY_ANDROID_T
            try
            {
                // вначале используем сохраненый токен для авторизации
                // если такой токен есть - то нам не потребуется повторный вызов
                // внешней авторизации
                var storedAuthenticator = new StoredPlayerAuthenticator();
                var player = await storedAuthenticator.Login(connector);
                if (player != null)
                {
                    resultText.text = "Login with StoredPlayerAuthenticator";
                }
                else
                {
                    // сохраненного токена нет или он не валиден
                    // необходима внешняя авторизация
                    try
                    {
                        var androidAuthenticator = new AndroidAuthenticator(androidWebClientId);
                        var result = await androidAuthenticator.LoginOrRegister(connector);
                        player = result.Player;
                        storedAuthenticator.Store(player);
                        resultText.text = "Login with AndroidPlayerAuthenticator";
                    }
                    catch (AndroidAuthenticateException e)
                    {
                        resultText.text = "Android API Error";
                        Debug.LogError(e.Message);
                    }
                }
            }
            catch (RpcException e)
            {
                resultText.text = "RPC Exception";
                Debug.LogError(e.Message);
            }

#endif
        }
    }
}